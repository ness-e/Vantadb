---
title: "S7 — Deep Audit: READMEs (raíz + sub-paquetes)"
kind: review
status: active
description: "Informe crudo del subagente S7: todos los READMEs (raíz, ES, python, ts, node, wasm, server, mcp, proxy, providers, integrations) contra la implementación real. Base de los hallazgos `S7-*` del ledger (40 hallazgos, el área más grande de la auditoría)."
tags: [audit, reviews, readme, docs, subagent]
---

# Informe del subagente — Deep Audit: README files (root + sub-packages)

**Repo:** `C:\Users\Eros\VantaDB Proyect\VantaDB`
**Auditor:** vanta-review (P2-01 second opinion, read-only)
**Date:** 2026-10-20
**Working tree:** branch `release-plz-fix-cargo-token` @ `2e8152a`; truth branch = `origin/main` = `origin/develop` = `bb12f2c`
**Scope:** `README.md`, `README_ES.md`, `docs/user/QUICKSTART.md`, and the READMEs of `vantadb-python/`, `vantadb-node/`, `vantadb-ts/`, `vantadb-wasm/`, `vanta-proxy/`, `vantadb-server/`, `providers/*/`, `integrations/*/`.

**Severity scale:** P0 = install path impossible / total breakage · P1 = documented command or claim fails at runtime, or points at a non-existent artifact · P2 = documented API/anchor/claim is wrong or diverges · P3 = inaccuracy, drift, or missing coverage.

**Summary:** 0 P0 · 13 P1 · 16 P2 · 11 P3 = **40 findings**.

---

## P1 — Documented commands / claims that fail at runtime or point at nothing

### S7-01 | P1 | CLI syntax — root + ES README
- **Location:** `README.md:329`, `README.md:349`, `README.md:359`; `README_ES.md:314`, `README_ES.md:334`, `README_ES.md:344`
- **Claim/Break:** The usage guide documents `vanta-cli server start --port 8080` (and `server start --host 0.0.0.0 --port 8080`). There is **no `start` subcommand**. `src/cli.rs:338` declares `Server { http: bool, mcp: bool, port, host, require_auth, allow_insecure, dashboard_dir }` — a flat clap variant, not `#[command(subcommand)]`. `grep -i "start" src/cli.rs` returns only doc comments (lines 337, 339, 343, 355, 360, 491) and `src/bin/*.rs` has zero matches. The correct invocation is `vanta-cli server --http --port 8080` — which the *same* README already states correctly at `README.md:310`.
- **Why it matters:** `clap` exits with `error: unexpected argument 'start'` — a first-run user copying the "Quick Start" block gets an immediate failure, twice, in both languages. It also contradicts the README's own NOTE block at `README.md:305-308` which declares `README.md § One-Line Installation` the source of truth.
- **Fix:** Replace `vanta-cli server start` with `vanta-cli server --http` in all six places (both READMEs, usage guide + prose + network-exposure block).
- **Verify:** `cargo run --bin vanta-cli -- server --help` (shows `--http/--mcp/--port/--host`, no `start`), and `cargo run --bin vanta-cli -- server start` → `error: unexpected argument 'start'`.

### S7-02 | P1 | Badge target — root + ES README
- **Location:** `README.md:9`, `README.md:10`; `README_ES.md:9`, `README_ES.md:10`
- **Claim/Break:** Two workflow badges target files that **do not exist on the default branch**: `gate-docs.yml` and `sec-codeql.yml`. `git ls-tree --name-only origin/main .github/workflows/` lists `certification.yml, ci-fleet.yml, ci-gate.yml, ci-rust.yml, demo-acceptance.yml, docs-gates.yml, electron-node.yml, nightly.yml, perf-bench.yml, platform-ops.yml, publish-mcp-registry.yml, release-adapters.yml, release-binaries.yml, release-npm-61.yml, release-npm-node.yml, release-providers.yml, release-sbom.yml, release-verify.yml, release-wheels.yml, release.yml`. The docs gate is `docs-gates.yml` (plus `ci-gate.yml`); there is no CodeQL workflow under that name. `ci-rust.yml` (L8) and `nightly.yml` (L11) **do** resolve.
- **Why it matters:** Both badges render "no status" permanently — a permanent red/false signal to every visitor of the repo landing page, on the two most trust-bearing badges (Docs, Security Audit).
- **Fix:** Point L9 at `docs-gates.yml` (or `ci-gate.yml`) and either drop L10 or repoint it at the workflow that actually runs CodeQL (none exists → remove the badge).
- **Verify:** `git ls-tree --name-only origin/main .github/workflows/ | grep -E "docs|codeql"`.

### S7-03 | P1 | Link to non-existent workflow — root + ES README
- **Location:** `README.md:219`, `README_ES.md:218`
- **Claim/Break:** "SIFT-1M remains useful as a stress/recovery scenario via the [Heavy Certification](https://github.com/ness-e/Vantadb/actions/workflows/heavy-certification.yml) workflow." No `heavy-certification.yml` exists on `origin/main`; the workflow is `certification.yml`. The GitHub URL 404s (the legacy `heavy-certification-50.yml` name existed only on the stale local `main` branch, `26005c8`).
- **Why it matters:** A dead link in the "Search Semantics" section, in the flagship repo, pointing readers at the certification evidence that is supposed to back the SIFT-1M claims in the same document.
- **Fix:** Change the URL to `.../actions/workflows/certification.yml`.
- **Verify:** `git ls-tree --name-only origin/main .github/workflows/ | grep cert`.

### S7-04 | P1 | Non-existent CI workflow cited as evidence — root + ES README
- **Location:** `README.md:139`, `README_ES.md:138`
- **Claim/Break:** "Each example … is exercised end-to-end by the CI example smoke suite (`ci-examples.yml`)." No workflow named `ci-examples.yml` exists on `origin/main` or in the working tree. The examples **are** run by a different workflow: `.github/workflows/ci-fleet.yml` lines 162-180 run `python examples/python/{agent_memory,langchain_ollama_rag,autogen_memory,langgraph_checkpoint,crewai_memory,dspy_retriever,haystack_documentstore,mem0_integration,semantic_kernel_memory}.py`.
- **Why it matters:** The README cites a *verification* artifact by name. A reviewer or user who checks the citation finds nothing, and the whole "exercised end-to-end by CI" claim loses its audit trail — while the claim itself is true under a different name.
- **Fix:** Replace `ci-examples.yml` with `ci-fleet.yml` in both files.
- **Verify:** `grep -n "examples/python" .github/workflows/ci-fleet.yml`.

### S7-05 | P1 | Non-existent workflow reference — integrations
- **Location:** `integrations/README.md:48`
- **Claim/Break:** "Techos elegidos contra la matriz de compatibilidad (`.github/workflows/adapters-compat.yml`, que fija minors: dspy 2.6, haystack 2.10, openai 1.40.)". There is no `adapters-compat.yml` in `.github/workflows/` (see S7-02 for the full list) and none in the working tree. The compat matrix that actually exists is `release-adapters.yml` / `ci-fleet.yml`.
- **Why it matters:** This is the stated justification for every framework upper-bound in the pins table (L35-45) — i.e. the entire "FIND-84 pins" contract rests on a file that isn't there. The pins themselves are real (verified in the adapters' `pyproject.toml`), but the rationale is unciteable.
- **Fix:** Repoint to the workflow that runs the compat matrix, or state the minors without the file reference.
- **Verify:** `ls .github/workflows | grep -i adapter`.

### S7-06 | P1 | Non-existent task file — vantadb-node
- **Location:** `vantadb-node/README.md:15`
- **Claim/Break:** "…y `docs/dev/tasks/TS-12.md` para la checklist del publish humano." `Test-Path docs/dev/tasks/TS-12.md` → **False**; a recursive `Get-ChildItem -Filter TS-12.md` over the whole repo returns **nothing**. `docs/dev/tasks/` holds `TS-09.md`, `TS-13.md`, `ERR-TS-01.md` and numeric ids. The task exists only in archived prose (`docs/CHANGELOG.md:2360`, `docs/dev/avance/activo/ci-cd.md:316`).
- **Why it matters:** The publish checklist is the only "how do I ship this" pointer for a package that is *not yet on npm*. The reader lands on a 404.
- **Fix:** Point at the archive location of TS-12 (e.g. `docs/dev/avance/activo/ci-cd.md` §TS-12) or inline the checklist.
- **Verify:** `Get-ChildItem -Recurse -Filter TS-12.md` → empty.

### S7-07 | P1 | False "not on PyPI" claim — integrations index (7 adapters)
- **Location:** `integrations/README.md:12-14`
- **Claim/Break:** "Los 7 paquetes publicables … **ninguno esta en PyPI** (verificado live 2026-10-06: `GET https://pypi.org/pypi/<nombre>/json` = 404 x7, y lo mismo en test.pypi.org…)". Live check today (2026-10-20) returns **HTTP 200 for all seven**:
  `vantadb-crewai` 0.5.0 · `vantadb-dspy` 0.5.0 · `vantadb-haystack` 0.5.0 · `vantadb-langchain` 0.5.0 · `vantadb-letta` 0.5.0 · `vantadb-llamaindex` 0.5.0 · `vantadb-mem0` 0.5.0.
- **Why it matters:** The verification claim ("verificado live 2026-10-06, 404 x7") is now false and, worse, is stated as current fact. It also drives the whole "publicar sin mantenedor no es salida valida (Stop del plan)" decision on L15-17, so a reader concludes the install path is source-only when `pip install vantadb-crewai` in fact works today.
- **Fix:** Rewrite the status block: the 7 adapters **are** on PyPI at 0.5.0 (released after 2026-10-06); state the current versions and the maintainer caveat separately from the "not yet published" history.
- **Verify:** `curl -s https://pypi.org/pypi/vantadb-crewai/json | head -c 200` → 200 + `"version":"0.5.0"`.

### S7-08 | P1 | False "Not on PyPI yet" banner — 7 adapter READMEs
- **Location:** `integrations/crewai/README.md:7-9`, `integrations/dspy/README.md:7-9`, `integrations/haystack/README.md:7-9`, `integrations/langchain/README.md:7-9`, `integrations/letta/README.md:7-9`, `integrations/llamaindex/README.md:7-9`, `integrations/mem0/README.md:7-9`
- **Claim/Break:** Each banner states "**Not on PyPI yet.** The `vantadb-<adapter>` package builds and passes `twine check` locally; it will go live with the first `adapters-v*` tag release. Until then, install from source." All seven distributions are on PyPI at 0.5.0 (same live check as S7-07).
- **Why it matters:** The banner directly contradicts the "### Install from PyPI (after first release)" section that follows it in the same file — and the `pip install vantadb-<adapter>` command it gates is now a working install. Users are told to `cd integrations/<x> && pip install .` when PyPI is the better path, and `pip install vantadb-crewai` in fact resolves to a *different* (0.5.0) build than the source tree (0.9.0-era), which is itself a version-skew hazard worth documenting.
- **Fix:** Replace with "Published on PyPI since <date> at 0.5.0 — `pip install vantadb-<adapter>` installs that release; install from source for the current tree."
- **Verify:** `GET https://pypi.org/pypi/vantadb-{crewai,dspy,haystack,langchain,letta,llamaindex,mem0}/json` → all 200.

### S7-09 | P1 | False "Not on PyPI yet" banner — 3 provider READMEs
- **Location:** `providers/litellm/README.md:7-9`, `providers/ollama/README.md:7-9`, `providers/openai/README.md:7-9`
- **Claim/Break:** "**Not on PyPI yet.** The `vantadb-litellm` wheel … goes live with the first `providers-v*` tag release." Live: `vantadb-litellm` → **200, 0.8.0**; `vantadb-ollama` → **200, 0.8.0**; `vantadb-openai` → **200, 0.8.0**.
- **Why it matters:** Same class as S7-08, plus a concrete trap: the PyPI name `vantadb-ollama` / `vantadb-openai` is **also** the name of the superseded Python integration (S7-11), so "not published" was the only thing keeping the two apart. With both published, the install path in `integrations/ollama/README.md:22` now resolves to the Rust provider while the quickstart next to it targets the Python adapter's API.
- **Fix:** Update the banners to the published state and versions.
- **Verify:** `GET https://pypi.org/pypi/vantadb-litellm/json` → 200, version 0.8.0.

### S7-10 | P1 | Example API does not exist — integrations/mem0
- **Location:** `integrations/mem0/README.md:32-35` (quickstart), `integrations/mem0/README.md:41,43` (API list)
- **Claim/Break:** The quickstart calls `store.search("vector database")` and iterates `r["payload"], r["score"]`; the API list says `search(query, k=4)` and `list(user_id=None, limit=100)`. The implementation is `integrations/mem0/vantadb_mem0/vectorstore.py:172` → `search(self, query: str, vectors: List[float], top_k: int = DEFAULT_TOP_K, filters=None)` — `vectors` is a **required positional** argument (mem0 computes embeddings externally and passes them in). And `list` (line 278) is `list(self, filters=None, top_k=100)`, not `list(user_id=None, limit=100)`. Finally `OutputData` (line 14) declares `__slots__ = ("id","score","payload")` with **no `__getitem__`**, so `r["payload"]` raises `TypeError: 'OutputData' object is not subscriptable`.
- **Why it matters:** Two of the three lines in the quickstart raise at runtime. The documented `search()` signature also misrepresents the mem0 contract (mem0 owns embedding; the adapter does not embed).
- **Fix:** Either document the real signature (`search(query, vectors, top_k=…)`, `list(filters=None, top_k=100)`) and use attribute access (`r.payload`, `r.score`), or add a `__getitem__` to `OutputData` and a convenience overload for `search`.
- **Verify:** `grep -n "def search\|def list\|__slots__" integrations/mem0/vantadb_mem0/vectorstore.py`.

### S7-11 | P1 | Install command produces an incompatible API — integrations/ollama & /openai
- **Location:** `integrations/ollama/README.md:22` vs `integrations/ollama/README.md:28-38`; `integrations/openai/README.md:22` vs `integrations/openai/README.md:28-39`
- **Claim/Break:** The "From PyPI" section says `pip install vantadb-ollama   # installs the Rust provider (providers-v* lane)`, and the quickstart immediately below does `from vantadb_ollama import VantaDBOllama` → `VantaDBOllama(model=…, db_path=…)` → `store.add_texts([...])` → `store.similarity_search(...)` → `doc.page_content`. The **Rust provider** module `vantadb_ollama` (see `providers/ollama/vantadb_ollama.pyi`) exports `VantaDBOllama(db_path, base_url, model, namespace, timeout)` with `embed` / `store` / `search` / `get` / `list` / `delete` / `list_namespaces` — **no `add_texts`, no `similarity_search`, no `page_content`**. The LangChain-style API belongs to the *Python integration* (`integrations/ollama/vantadb_ollama/vectorstore.py:15`, `class VantaDBOllama(EmbeddingVectorStore)`), which is source-only. Same for `vantadb-openai` / `VantaDBOpenAI`.
- **Why it matters:** Following the README end-to-end installs the provider and then runs a quickstart that raises `TypeError: VantaDBOllama.__init__() got an unexpected keyword argument 'model'` (and then `AttributeError: 'VantaDBOllama' object has no attribute 'add_texts'`). The two artifacts share a PyPI name **and** a module name — the exact collision FIND-273 was meant to prevent — and the README documents the colliding path as the happy path.
- **Fix:** Split the two quickstarts explicitly: "Rust provider (PyPI) → `embed`/`store`/`search`" and "Python adapter (source-only) → `add_texts`/`similarity_search`", and stop advertising `pip install vantadb-ollama` in a file whose quickstart targets the other artifact.
- **Verify:** Compare `providers/ollama/vantadb_ollama.pyi` with `integrations/ollama/vantadb_ollama/vectorstore.py`.

### S7-12 | P1 | Broken anchor — vantadb-python + vantadb-ts
- **Location:** `vantadb-python/README.md:120`, `vantadb-ts/README.md:237`
- **Claim/Break:** Both link to `../docs/user/QUICKSTART.md#4-real-embeddings-optional`. `docs/user/QUICKSTART.md` has **no** section 4 titled "Real Embeddings" — §4 is "Install the Python Binding from Source" (L105) and the embeddings section is `## 7. Optional: Local Embeddings (\`embed-local\`)` (L251), whose GitHub anchor is `#7-optional-local-embeddings-embed-local`. The target anchor does not exist.
- **Why it matters:** The "full walkthrough" link — the single pointer from both SDK READMEs to the real-embedding guide — 404s to a section that doesn't exist. This is the highest-traffic cross-link in the two SDK READMEs.
- **Fix:** Point to `#7-optional-local-embeddings-embed-local`, or add a `## 4. Real Embeddings` section to QUICKSTART.md and renumber.
- **Verify:** `grep -n "^## " docs/user/QUICKSTART.md` → no "Real Embeddings" heading.

### S7-13 | P1 | EN/ES divergence on a user-visible behaviour
- **Location:** `README.md:70` vs `README_ES.md:66-68`
- **Claim/Break:** EN: "`import vantadb_py` still works but emits a `DeprecationWarning`." ES: "`import vantadb_py` sigue disponible sin cambios." ("still available, unchanged"). The code emits the warning: `vantadb-python/vantadb_py/__init__.py` calls `warnings.warn("The 'vantadb_py' import name is deprecated and will be removed in the next minor release (0.6.0)…", DeprecationWarning, stacklevel=2)`. The ES wording asserts the opposite of the EN wording *and* of the code.
- **Why it matters:** A Spanish-speaking user is told the legacy import is unchanged and therefore keeps using it, and is never warned that the path is scheduled for removal. This is exactly the class of defect the brief calls out ("a translated README asserting a different number/version is a real bug") — here a different *behaviour*.
- **Fix:** Mirror the EN sentence in ES: "`import vantadb_py` sigue funcionando pero emite un `DeprecationWarning`."
- **Verify:** `grep -n "DeprecationWarning" vantadb-python/vantadb_py/__init__.py`.

---

## P2 — Documented API / anchor / claim wrong or divergent

### S7-14 | P2 | Wrong source path — root + ES README
- **Location:** `README.md:208`, `README_ES.md:207`
- **Claim/Break:** "Python bindings route through a stable `src/sdk.rs` boundary." There is no `src/sdk.rs` file — `src/sdk` is a **directory** (`api.rs, builder.rs, connect.rs, gds.rs, graph.rs, merge.rs, mod.rs, types.rs, version_history.rs` plus `api/`, `importers/`, `search/`, `serialization/`, `types/`). The same stale `src/sdk.rs` reference appears in the crate's own `vantadb-node/dts-header.d.ts` header, so it is a propagated typo.
- **Why it matters:** Points readers at a file that cannot be opened; the actual boundary is `src/sdk/mod.rs` (+ `src/sdk/api.rs`).
- **Fix:** Write `src/sdk/` (module) or `src/sdk/api.rs`.
- **Verify:** `Get-ChildItem src/sdk -Name` / `Test-Path src/sdk.rs` → False.

### S7-15 | P2 | Broken TOC anchor — root + ES README
- **Location:** `README.md:50`, `README_ES.md:46`
- **Claim/Break:** Quick Links point to `#optional-server-mode` (EN) and `#modo-servidor-opcional` (ES). The actual headings are `## Server Mode & Self-Healing Daemon` (`README.md:347`) and `## Modo Servidor y Demonio Self-Healing` (`README_ES.md:332`), whose anchors are `#server-mode--self-healing-daemon` and `#modo-servidor-y-demonio-self-healing`. Neither Quick Links target exists. (All other Quick Links anchors were checked and resolve: `#product-boundary--one-product-four-surfaces`, `#installation`, `#embedded-cli`, `#documentation`, `#límite-del-producto--un-producto-cuatro-superficies`, `#instalación`, `#cli-embebida`, `#documentación`.)
- **Why it matters:** The "Run as a local server" entry in the very first table of the README lands on a non-existent anchor (GitHub silently lands at the top of the page).
- **Fix:** Point at the real heading anchors.
- **Verify:** Compare the Quick Links targets with the GitHub slugs of the `##` headings.

### S7-16 | P2 | Missing section in the Spanish translation
- **Location:** `README.md:240-248` (present) vs `README_ES.md` (absent — between `### Notas de plataforma` at L239 and `## CLI embebida` at L246)
- **Claim/Break:** The EN README has a `### North Star` section (metric definition, the `python scripts/north_star_metric.py --db <proxy-db-path> [--days 7] [--json]` command, and the "Status: first measurement pending" statement) and the Product-Boundary table links to it (`README.md:230` → `#north-star`). The ES README has **no** North Star section and its Product-Boundary row (`README_ES.md:229`) mentions "la North Star" with no link and no definition. `scripts/north_star_metric.py` exists (9059 bytes) and is unreferenced from the ES README.
- **Why it matters:** The ES README is a strict mirror by design (same 20 sections, same tables). A whole subsection — the product's headline metric — is missing from the translation, and the ES text gestures at a metric it never defines.
- **Fix:** Translate the North Star section into README_ES.md and link the MCP row to it.
- **Verify:** Diff the `##`/`###` heading lists of both files (currently EN has 1 extra `###`).

### S7-17 | P2 | EN/ES divergence on the North Star definition
- **Location:** `README.md:230` vs `README_ES.md:229`
- **Claim/Break:** EN: "This is where the North Star is measured: **sessions with put + search** ([details](#north-star))." ES: "Acá se mide la North Star: **sesiones con recall exitoso**." — "recall exitoso" is a *different metric definition* (successful recall vs. put+search), and the anchor is gone.
- **Why it matters:** Two published READMEs stating two different definitions of the product's single headline metric. `docs/api/PROXY.md:233` confirms the canonical wording: "*sessions with put + search in a 7-day window*".
- **Fix:** Use "sesiones con put + search" in ES and restore the link.
- **Verify:** `docs/api/PROXY.md:231-234`.

### S7-18 | P2 | EN/ES divergence on the install wizard
- **Location:** `README.md:288-295` vs `README_ES.md:277-280`
- **Claim/Break:** EN describes the wizard as asking "**which modules to enable** — motor, MCP, server, proxy, desktop viewer, embeddings, providers (each with a declared state: installable or frozen; desktop is a frozen viewer)" and documents `--modules mcp,embeddings` / `-Modules mcp,embeddings` / `--no-wizard` / `-NoWizard` / `--dry-run` / `-DryRun`. ES describes it as "(`setup-embeddings.ps1` — modelo, bloque MCP por cliente, proxy activado por defecto) salvo que se omita con `--no-wizard`…" and **omits** the `--modules` / `-Modules` / `--dry-run` / `-DryRun` flags entirely. The scripts match the **EN** text: `scripts/install.sh:48` — "`--modules <csv>`: wizard module selection (engine,mcp,server,proxy,desktop,embeddings,providers; omitted = wizard defaults mcp,server,proxy,embeddings)"; `scripts/install.ps1:10-12` mirrors it.
- **Why it matters:** The ES reader is told the wizard picks an embedding model and an MCP block per client (a behaviour that does not exist), and is not told the scripted-selection flags that the EN reader gets. Two languages, two different installers.
- **Fix:** Translate the EN block verbatim, including `--modules`/`-Modules`, `--no-wizard`/`-NoWizard`, `--dry-run`/`-DryRun`.
- **Verify:** `grep -n "modules\|no-wizard\|dry-run" scripts/install.sh scripts/install.ps1`.

### S7-19 | P2 | Wrong exported class name — vantadb-ts
- **Location:** `vantadb-ts/README.md:245`
- **Claim/Break:** The Lifecycle table says "| `VantaDB.create(config?)` | Create in-memory or configured instance |". The package exports `Client` — `vantadb-ts/src/vantadb.ts:181` `export class Client`, `:247` `static create(config?: Config): Client`, `:273` `static open(path: string): Client`. There is **no `VantaDB` export**: `grep "VantaDB" vantadb-ts/src/*.ts` only matches `VANTADB_*` error codes and doc text; the module's export block (`vantadb.ts:1765-1773`) re-exports `DbError, ERROR_CODES`, the guards, and `* from "./native.js"`. Every other row of the same table correctly says `Client.open(path)`.
- **Why it matters:** A reader looking for the entry point finds a class that cannot be imported (`import { VantaDB } from "vantadb"` → `SyntaxError: The requested module 'vantadb' does not provide an export named 'VantaDB'`).
- **Fix:** `Client.create(config?)`.
- **Verify:** `grep -n "export class\|static create" vantadb-ts/src/vantadb.ts`.

### S7-20 | P2 | Wrong exported class name — vantadb-wasm
- **Location:** `vantadb-wasm/README.md:119-121`, `:154`, `:158`
- **Claim/Break:** Three code blocks use `import { VantaDB } from "vantadb"; const db = VantaDB.create();` and `const db = VantaDB.create(); // throws if window is undefined`. The `vantadb` package exports `Client`, not `VantaDB` (see S7-19). Line 210 in the same file uses the third spelling, `const db = new Client();`, and `Client`'s constructor is **`private`** (`vantadb-ts/src/vantadb.ts:185` `private constructor(inner: WasmClient)`) — so `new Client()` is a TypeScript compile error ("Constructor of class 'Client' is private"), even though it happens to work at runtime in plain JS.
- **Why it matters:** Three different, all-wrong spellings of the entry point in one README, including a private-constructor call. This is the "how do I start" reference for the browser connector.
- **Fix:** `import { Client } from "vantadb"; const db = Client.create();` everywhere; delete the `new Client()` form.
- **Verify:** `grep -n "private constructor\|static create" vantadb-ts/src/vantadb.ts`.

### S7-21 | P2 | Runtime-support table contradicts the package's own `engines`
- **Location:** `vantadb-ts/README.md:381-382` vs `vantadb-ts/package.json:6-8` and `vantadb-node/README.md:25`
- **Claim/Break:** The Runtimes table claims "Node.js 18–22.11 | ⚠️ ESM only — use `import`" — i.e. Node 18 works. `vantadb-ts/package.json` declares `"engines": { "node": ">=22.19" }`, and `vantadb-node/README.md:25` states the same: "Compat: `vantadb-ts` exige Node ≥ 22.19 (`vantadb-ts/package.json:engines`)". The TS README's own §"Module Formats" says `require()` needs "Node.js ≥ 22.12" — a third number, 22.12, distinct from both 22.19 and the table's 18.
- **Why it matters:** The published npm package refuses (or warns) to install below 22.19, while the README tells Node 18 users to use it. Two of the three numbers in the same file are wrong.
- **Fix:** Make the Runtimes table respect `engines` (Node ≥ 22.19 supported; 18–22.18 unsupported), and reconcile the 22.12 figure with the 22.19 `require(esm)` floor.
- **Verify:** `node -e "console.log(require('./vantadb-ts/package.json').engines)"`.

### S7-22 | P2 | Wrong directory in a runnable command — vantadb-python
- **Location:** `vantadb-python/README.md:165`
- **Claim/Break:** "```bash\n# Format Python code\nblack tests/ vantadb_python/\n```". The package directory is `vantadb_py/` (underscore), matching `pyproject.toml [tool.maturin] module-name = "vantadb_py"`. There is no `vantadb_python/` directory (`Get-ChildItem vantadb-python -Name` → `src, vantadb, vantadb_py, tests, …`).
- **Why it matters:** The command fails with "No such file or directory" — a broken dev-loop instruction in the Development section.
- **Fix:** `black tests/ vantadb_py/`.
- **Verify:** `Test-Path vantadb-python/vantadb_python` → False; `Test-Path vantadb-python/vantadb_py` → True.

### S7-23 | P2 | Wrong ADR id — vantadb-python
- **Location:** `vantadb-python/README.md:25`
- **Claim/Break:** "**Naming (ADR-041 anti-stutter):** the canonical client name is `Client`…". `docs/dev/architecture/adr/ADR-0041-error-variant-renames.md` is the **error-variant renames** ADR; the anti-stutter ADR is `ADR-0047-anti-stutter.md` ("ADR-0047: Anti-stutter total (eliminar prefijo Vanta redundante)"), which is exactly what the root README cites (`README.md:72`).
- **Why it matters:** Cites the wrong decision record; a reader chasing the naming rationale lands on an unrelated ADR.
- **Fix:** `ADR-0047 anti-stutter`.
- **Verify:** `ls docs/dev/architecture/adr | grep -E "0041|0047"`.

### S7-24 | P2 | Documented signature does not exist — integrations/haystack
- **Location:** `integrations/haystack/README.md:45`
- **Claim/Break:** "`count_documents(filters)` - count documents". The implementation is `integrations/haystack/vantadb_haystack/vectorstore.py:371` → `def count_documents(self) -> int:` — **no** `filters` parameter. Calling `count_documents(filters={...})` raises `TypeError`.
- **Why it matters:** A documented argument that raises at runtime, in the API table of a published adapter.
- **Fix:** Either drop `filters` from the doc, or add the parameter to the implementation (Haystack's `DocumentStore` protocol takes no filters either, so dropping it is the correct move).
- **Verify:** `grep -n "def count_documents" integrations/haystack/vantadb_haystack/vectorstore.py`.

### S7-25 | P2 | Documented signatures do not exist — integrations/mem0 (API table)
- **Location:** `integrations/mem0/README.md:41`, `integrations/mem0/README.md:43`
- **Claim/Break:** "L41 `search(query, k=4)` - search memories" and "L43 `list(user_id=None, limit=100)` - list memories". Real signatures: `search(self, query: str, vectors: List[float], top_k: int = DEFAULT_TOP_K, filters=None)` (`vectorstore.py:172`) and `list(self, filters: Optional[Dict[str, Any]] = None, top_k: int = 100)` (`vectorstore.py:278`). There is no `k` parameter and no `user_id` parameter.
- **Why it matters:** Same defect class as S7-10 but in the API table; a user porting mem0 code will call `search(query, k=4)` and get `TypeError: search() got an unexpected keyword argument 'k'`.
- **Fix:** Document the real signatures (and note that mem0 supplies the embedding vector).
- **Verify:** `grep -n "def search\|def list" integrations/mem0/vantadb_mem0/vectorstore.py`.

### S7-26 | P2 | Documented return shape does not exist — providers/ollama
- **Location:** `providers/ollama/README.md:49`
- **Claim/Break:** "| `list` | `list(namespace, limit=100, cursor=None) -> dict` | Paginated listing: `{"records": [...], "cursor": str}` when a next page exists. |" The implementation sets **`next_cursor`**, not `cursor`: `providers/ollama/src/python.rs:321` → `d.set_item("next_cursor", page.next_cursor)?;`. The two sibling READMEs say it correctly (`providers/litellm/README.md:49`, `providers/openai/README.md:49` → `{"records": [...], "next_cursor": int | None}`), and the .pyi confirms the same for all three.
- **Why it matters:** Copy-paste drift inside a 3-file family: the ollama row documents a key that is never emitted, so `result["cursor"]` → `KeyError`.
- **Fix:** Change the ollama row to `{"records": [...], "next_cursor": int | None}`.
- **Verify:** `grep -n "next_cursor" providers/ollama/src/python.rs providers/litellm/src/python.rs providers/openai/src/python.rs`.

### S7-27 | P2 | Wrong section reference — vantadb-wasm
- **Location:** `vantadb-wasm/README.md:327`
- **Claim/Break:** "‑ `docs/user/QUICKSTART.md` §"4. Real Embeddings" - full walkthrough". `docs/user/QUICKSTART.md` has no "Real Embeddings" section; §4 is "Install the Python Binding from Source" and the embeddings material is §7 "Optional: Local Embeddings (`embed-local`)" (L251). This is the same stale target as S7-12, expressed as prose rather than a link.
- **Why it matters:** The "References" list of the WASM README points at a non-existent section, and the TS/Python READMEs point at a non-existent *anchor* of the same phantom section — one drift with three symptoms.
- **Fix:** Point to `docs/user/QUICKSTART.md §7` (or add the missing `## 4. Real Embeddings` section once and fix all three).
- **Verify:** `grep -n "^## " docs/user/QUICKSTART.md`.

### S7-28 | P2 | A canonical name in the root README is not importable in Python
- **Location:** `README.md:74`, `README_ES.md:71`
- **Claim/Break:** "canonical names are `Client` (legacy `VantaDB` alias removed in 0.6.0, AST-010), `Record`, `SearchHit`, **`Config`**." In the Python SDK, `Config` does not exist: `grep -w Config` over `vantadb-python/vantadb_py/vantadb_py.pyi`, `vantadb_python_py/__init__.py`, `vantadb/__init__.py`, `vantadb/__init__.pyi` returns **zero** matches; the only `Config` in the crate is the internal Rust `vantadb::config::Config` (`vantadb-python/src/lib.rs:15,155`). `Config` is a **TypeScript** type (`vantadb-ts/src/vantadb.ts:247` `static create(config?: Config)`), not a Python name. The note is otherwise explicitly about Python (`db.memory`, `db.search`, `db.search_vector`, AST-008/AST-012).
- **Why it matters:** `from vantadb import Config` → `ImportError`. The Python SDK's own README lists a *different* canonical set for the same concept ("`Record`, `SearchHit`/`Hit`, `ListResult`, `Vector`" — `vantadb-python/README.md:26-27`), so the two READMEs disagree about the canonical vocabulary.
- **Fix:** Drop `Config` from the Python naming note (or scope it explicitly to the TypeScript SDK), and reconcile the two canonical-type lists.
- **Verify:** `grep -w "Config" vantadb-python/vantadb_py/vantadb_py.pyi` → no match.

---

## P3 — Inaccuracies, drift, and coverage gaps

### S7-29 | P3 | EN/ES divergence — storage backend name
- **Location:** `README.md:201` vs `README_ES.md:200`
- **Claim/Break:** EN: "`StorageBackend` + **File** + WAL"; ES: "`StorageBackend` + **VantaFile** + WAL". `VantaFile` is a real internal type (referenced in `docs/user/operations/CONFIGURATION.md:75`), so the ES is not inventing a term — it is naming a *different* layer than the EN table's "File". The two tables purport to describe the same mechanism.
- **Why it matters:** A reader comparing the two tables sees two different storage backends.
- **Fix:** Use "VantaFile" in both (or "File (VantaFile)").
- **Verify:** `grep -rn "VantaFile" src/storage --include=*.rs | head`.

### S7-30 | P3 | ES README is missing a badge
- **Location:** `README_ES.md:9-10` (vs `README.md:8-11`)
- **Claim/Break:** The EN README carries four workflow badges (Rust CI, Docs, Security Audit, Nightly); the ES carries three — the **Nightly** badge (`README.md:11`) has no Spanish counterpart.
- **Why it matters:** The two landing pages present a different set of CI signals; the ES page under-reports the project's automated coverage.
- **Fix:** Add the Nightly badge block to README_ES.md.
- **Verify:** Diff the badge blocks of the two files.

### S7-31 | P3 | Wrong SDK import name in prose — root + ES README
- **Location:** `README.md:139`, `README_ES.md:138`
- **Claim/Break:** "Each example defines a thin wrapper class over the stable Python SDK (`vantadb_py`)". All three example files import the **canonical** name: `examples/python/mem0_integration.py:1` `import vantadb`, `semantic_kernel_memory.py:1` `import vantadb`, `dspy_retriever.py:1` `import vantadb`.
- **Why it matters:** Points readers at the deprecated import name (see S7-13) as the recommended one.
- **Fix:** Say "the canonical Python SDK (`vantadb`)".
- **Verify:** `grep -n "^import vantadb" examples/python/*.py`.

### S7-32 | P3 | Wrong positional signature — integrations/crewai
- **Location:** `integrations/crewai/README.md:44`
- **Claim/Break:** "`VantaDBTool(name, description, db_path, namespace)`". The real signature (`integrations/crewai/vantadb_crewai/vectorstore.py:32`) is `__init__(self, embedding=None, name="VantaDB Search", description="Search documents stored in VantaDB", *, db_path="./vantadb_data", namespace=DEFAULT_NAMESPACE, top_k=…, memory_limit_bytes=…, read_only=…, backend=…)` — `embedding` is first, and `db_path`/`namespace` are **keyword-only**. Positional use as documented raises `TypeError`.
- **Why it matters:** The quickstart below it uses keywords (and is correct), so the API table and the quickstart disagree.
- **Fix:** `VantaDBTool(embedding=None, name=…, description=…, *, db_path=…, namespace=…)`.
- **Verify:** `grep -n -A10 "def __init__" integrations/crewai/vantadb_crewai/vectorstore.py`.

### S7-33 | P3 | Overstated "configurable" — root + ES README
- **Location:** `README.md:203`, `README_ES.md:202`
- **Claim/Break:** "Native HNSW | Cosine similarity with configurable `M`, `ef_construction`, and `ef_search`." The three parameters exist as `HnswParams` fields (`src/index/graph/types.rs:108,110,111`) but are **not** exposed in the user-facing `Config` (`grep -n "HnswParams\|hnsw_params" src/config.rs` → 0 matches) nor in `docs/user/operations/CONFIGURATION.md`. The only user-facing knobs are `mmap_hnsw` (`src/config.rs:240`) and `flat_threshold` (`src/config.rs:922`, `VANTADB_FLAT_THRESHOLD`).
- **Why it matters:** A user looking for a way to tune `ef_search` in config will not find one; the tuning entry point is the Rust `HnswParams` struct.
- **Fix:** Say "tunable at the Rust API level (`HnswParams`)".
- **Verify:** `grep -rn "ef_search" src/config.rs docs/user/operations/CONFIGURATION.md` → no hits.

### S7-34 | P3 | "Hybrid Search" label on a vector-only call — root + ES README
- **Location:** `README.md:121-122`, `README_ES.md:120-121`
- **Claim/Break:** The quickstart labels `hits = db.search("agent/main", query_vector=[…], top_k=5)` as "**Hybrid Search** (BM25 + Cosine Similarity fused via RRF)". With no `text_query`, the engine's planner takes the vector-only arm: `src/sdk/search/debug_ops.rs:241-245` matches `(Some(text_query), true, _) => hybrid`, and a `None` text query falls through to the dense-only branch. `docs/user/operations/BENCHMARKS.md:62` makes the same point about the synthetic corpus ("the hybrid row above is likewise dominated by its vector leg").
- **Why it matters:** The flagship quickstart mislabels the operation it demonstrates; a reader copying it gets vector-only ANN and expects BM25 fusion.
- **Fix:** Add `text_query="…"` to the example, or relabel the step "Vector search (add `text_query=` for hybrid BM25+RRF)".
- **Verify:** `grep -n -A6 "match (text_query, has_vector, query_sparse)" src/sdk/search/debug_ops.rs`.

### S7-35 | P3 | Bare filename instead of a path — vantadb-server
- **Location:** `vantadb-server/README.md:5`
- **Claim/Break:** "(Deployment: systemd o Kubernetes - ver `DEPLOYMENT_GUIDE.md`; docker retirado 2026-10-02.)". The file lives at `docs/user/operations/DEPLOYMENT_GUIDE.md`; there is no `DEPLOYMENT_GUIDE.md` at the package root or repo root. Being in backticks rather than a link, it is a soft reference, but it does not resolve from the file's own location.
- **Why it matters:** The deployment guide is the only operational reference in a 5-line README; the bare name sends the reader hunting.
- **Fix:** Use `../docs/user/operations/DEPLOYMENT_GUIDE.md` as a real link.
- **Verify:** `Get-ChildItem -Recurse -Filter DEPLOYMENT_GUIDE.md`.

### S7-36 | P3 | Path-ambiguous reference — vanta-proxy
- **Location:** `vanta-proxy/README.md:16`
- **Claim/Break:** "Full reference: `docs/api/PROXY.md`." From `vanta-proxy/`, that relative path does not exist; the file is at the repo root (`docs/api/PROXY.md`, 20863 bytes). It is in backticks, so it reads as a repo-root-relative path — ambiguous but not a broken link.
- **Why it matters:** Minor; the reader can find the file.
- **Fix:** `../docs/api/PROXY.md` as a link.
- **Verify:** `Test-Path docs/api/PROXY.md`.

### S7-37 | P3 | Missing package README — vantadb-mcp
- **Location:** `vantadb-mcp/` (no `README.md`; contents: `Cargo.toml`, `src/`, `tests/`)
- **Claim/Break:** The brief lists `vantadb-mcp/` among the sub-package READMEs to audit, and the root README positions MCP as a headline surface (`README.md:37`, `README.md:230` — "the door for AI IDEs and agents"). The package ships **no README**; its canonical doc is `docs/api/MCP.md` (49257 bytes) and the crate is a workspace member (`Cargo.toml:775`).
- **Why it matters:** npm/PyPI-style package READMEs are what a user sees on the package page; here the "Agent (MCP)" surface — one of the four product surfaces — has no package-level entry point. `vantadb-server/`, `vanta-proxy/`, `vantadb-node/` all have one.
- **Fix:** Add a short `vantadb-mcp/README.md` pointing at `docs/api/MCP.md` with the install/run instructions.
- **Verify:** `Get-ChildItem vantadb-mcp -Name` → no README.md.

### S7-38 | P3 | Stale pin in QUICKSTART — version claim
- **Location:** `docs/user/QUICKSTART.md` (the `> **Note**` under `## 0. Install without cloning`, ~L57-60)
- **Claim/Break:** "the Python examples in §5 use `vantadb-py>=0.8.0` from PyPI (`pip install vantadb-py==0.8.0` - verified 2026-10-06…)". The live PyPI version is **0.8.0** (`GET https://pypi.org/pypi/vantadb-py/json` → 200, `info.version = 0.8.0`) while the workspace is at **0.9.0** (`Cargo.toml` `[workspace.package] version = "0.9.0"`). The pin is therefore *current* but the README does not tell the reader that a newer source tree exists.
- **Why it matters:** Low impact, but the version numbers quoted across the repo (0.8.0 PyPI / 0.9.0 tree / 0.9.0 npm `package.json`) are never reconciled, which is how the S7-07/08/09 "not on PyPI" drift went unnoticed.
- **Fix:** Note both ("PyPI latest 0.8.0; this tree is 0.9.0 — not yet on PyPI").
- **Verify:** `GET https://pypi.org/pypi/vantadb-py/json` → 0.8.0; `grep -n 'version = "0.9.0"' Cargo.toml`.

### S7-39 | P3 | "Three active language connectors" not enumerated
- **Location:** `vantadb-node/README.md:3`, `vantadb-wasm/README.md:3`, `vantadb-ts/README.md:5`, `vantadb-python/README.md:3`
- **Claim/Break:** All four READMEs claim "active" status, and the node/wasm ones both call themselves "one of the **three** active language connectors". The set is never defined; the TS and WASM READMEs describe the same connector from two layers, so the count is only reconcilable by inference. The root README's product table (`README.md:227-232`) uses a different taxonomy entirely ("four surfaces").
- **Why it matters:** Three different counting schemes (4 surfaces / 3 connectors / 5+ active packages) across the same doc set, none of them cross-referenced.
- **Fix:** Define the set once (e.g. in the root README) and link to it.
- **Verify:** Compare the "Status" callouts in the four package READMEs.

### S7-40 | P3 | `search()` example missing `await` context
- **Location:** `vantadb-ts/README.md:206-214`
- **Claim/Break:** The error-handling example is `try { db.search({ namespace: "docs", query_vector: [0, 0, 0] }); } catch (err) { … }` — no `await`, and no `try`/`catch` around an `await`. The TS `search` is synchronous (`vantadb-ts/src/vantadb.ts:746` `search(request: SearchRequest): SearchHit[]`), so the `try/catch` does catch a throw — the omission of `await` is harmless — but every other example in the same README awaits `db.search(...)` (L15, L54, L231), so the file is internally inconsistent about the sync/async contract of the WASM SDK.
- **Why it matters:** A reader may conclude `search` is sync in one place and async in another, and `await` on a non-promise in the other examples is dead code.
- **Fix:** State the sync/async contract once and make the examples consistent (the WASM SDK is synchronous; `vantadb-node` is async).
- **Verify:** `grep -n "search(request: SearchRequest)" vantadb-ts/src/vantadb.ts` → returns `SearchHit[]`, not a Promise.

---

## Verified-correct (no finding)

Recorded so the audit is falsifiable, not just a list of complaints.

| Claim | Location | Evidence |
|---|---|---|
| `pip install vantadb-py` is the right distribution name | `README.md:67`, `README_ES.md:63`, `vantadb-python/README.md:20` | `vantadb-python/pyproject.toml` → `name = "vantadb-py"`; `GET https://pypi.org/pypi/vantadb-py/json` → 200 (0.8.0); `GET https://pypi.org/pypi/vantadb/json` → **404** (confirms `vantadb` is not the PyPI name) |
| `import vantadb` is canonical; `vantadb_py` is deprecated | `README.md:70` | `vantadb-python/vantadb/__init__.py` (alias, suppresses the warning); `vantadb-python/vantadb_py/__init__.py` (emits `DeprecationWarning`); neither exports a `VantaDB` class → the "legacy `VantaDB` alias removed in 0.6.0" claim holds |
| npm packages are `vantadb` (TS/WASM) and `vantadb-node` (native) | `README.md:79-80` | `vantadb-ts/package.json` → `name = "vantadb"`; `vantadb-node/package.json` → `name = "vantadb-node"`; npm: `vantadb` → 200 (0.8.0), `vantadb-wasm` → 200 (0.8.0), `vantadb-node` → **404** |
| Rust badge `1.94.1+` | `README.md:22` | `Cargo.toml` `[workspace.package] rust-version = "1.94.1"` (MSRV) — matches; `rust-toolchain.toml` pins `channel = "1.95.0"` (toolchain, not MSRV) |
| Python badge `3.11+` | `README.md:21` | `vantadb-python/pyproject.toml` → `requires-python = ">=3.11"` |
| License badge + "Apache 2.0" | `README.md:23,471` | `LICENSE` = Apache License 2.0; `Cargo.toml` `license = "Apache-2.0"`; `deny.toml:49` allows `Apache-2.0`; `vantadb-node/package.json` + `vantadb-ts/package.json` `license = "Apache-2.0"` |
| Root quickstart is runnable Python | `README.md:103-133` | `Client.__init__(db_path, memory_limit_bytes=…)`; `put(namespace,key,payload,metadata=,vector=)`; `memory.get(namespace,key)`; `search(namespace, query_vector=, top_k=)`; `hardware_profile()`; `flush()`; `close()` — all present in `vantadb-python/vantadb_py/vantadb_py.pyi:177,190,218,479,342,346,360` |
| Root quickstart label "Hybrid Search" — *see S7-34* | `README.md:121` | — |
| All 3 integration example snippets run | `README.md:145-184` | `examples/python/mem0_integration.py:33,98` (`add`,`search` → dicts with `score`/`content`), `semantic_kernel_memory.py:39,118` (`save_information`,`retrieve` → `relevance`/`text`), `dspy_retriever.py:22,40,116` (`__init__`,`add`,`__call__` → `text`/`score`) — all key names match |
| CI badges `ci-rust.yml` / `nightly.yml` resolve | `README.md:8,11` | present on `origin/main` |
| `pip install maturin` + `maturin develop --manifest-path vantadb-python/Cargo.toml` | `README.md:426-429` | `vantadb-python/Cargo.toml` exists; `pyproject.toml` requires `maturin>=1.15.0,<2.0` |
| `python benchmarks/vantadb_local_bench.py --size 10000 --dim 128 --queries 1000` | `README.md:396,434` | `benchmarks/vantadb_local_bench.py` (9963 bytes) exists; BENCHMARKS.md:76 documents the same command |
| "p50 1.2 ms / 6.1 ms / 4.88x / 0.9560 / 0.9980" | `README.md:392-394` | `docs/user/operations/BENCHMARKS.md:34,35,36,28,31` — exact match |
| SDK-scope "put 13.5 ms · HNSW 2.4 ms · hybrid RRF 5.6 ms" | `README.md:396` | `BENCHMARKS.md:56,58,59` → 13.510 / 2.372 / 5.593 — correct rounding |
| Full SIFT-1M Phase-2 table (5 rows, speedups, p99, QPS) | `README.md:404-410` | `BENCHMARKS.md:146-150` — all 40 cells match |
| "Prebuilt binaries already include the HTTP server feature" | `README.md:310` | `.github/workflows/release-binaries.yml:121` → `cargo build --release --features "server,$ALLOC_FEATURES" --bin vanta-cli` |
| `VANTADB_BACKEND=rocksdb` / `=memory`; Fjall default | `README.md:201` | `src/config.rs:1192-1198` (env var, values `rocksdb`/`memory`); `Cargo.toml` `[features] default = [... "fjall" ...]`, `rocksdb = ["dep:rocksdb"]` |
| `VANTADB_HOST` env var | `README.md:373` | `src/cli.rs:352` `#[arg(long, env = "VANTADB_HOST")]` |
| `vanta-cli config show / doctor / set <key> <value>` | `README.md:324-326` | `src/cli.rs:403-427` `ConfigCommand::{Show,Set,Doctor}` |
| `vanta-cli put/list/export/rebuild-index/audit-index/repair-text-index/verify/certificate verify` | `README.md:333-340` | `src/cli.rs` `Commands::{Put,List,Export,RebuildIndex,AuditIndex,RepairTextIndex,Verify,Certificate}`; `--db` and `--json` are global args (`src/cli.rs:15-38`); `CertificateCommand::Verify { file }` (`:511`) |
| `vanta-cli server --mcp` (without `start`) | `README.md:330` | `src/cli.rs:338-346` `mcp: bool` flag — valid |
| `scripts/install.sh` / `install.ps1` exist at `main` | `README.md:272,278` | `git ls-tree origin/main scripts/` → both present |
| `docs/api/PROXY.md#north-star-metric-icp-01` anchor | `README.md:248` | `docs/api/PROXY.md:231` heading `## North Star metric (ICP-01)` → slug matches |
| `vantadb-wasm/README.md#1-bundle-sizes-…` and `#4-honest-comparison-…` | `vantadb-ts/README.md:89,157,171` | `vantadb-wasm/README.md:24` and `:232` — slugs match |
| `BENCHMARKS.md#-5-impact-of-loop-and-hnsw-distance-optimization-phase-2` | `vantadb-ts/README.md:167`, `vantadb-wasm/README.md:257` | `BENCHMARKS.md:134` heading `## § 5. Impact of Loop and HNSW Distance Optimization (Phase 2)` → `§` is stripped leaving a leading space → `-5-impact-of-…` — **matches** |
| `docs/api/BINDINGS_NAMESPACES.md#sub-client-roadmap-post-d43--ts-11-reviewed-2026-10-04` | `vantadb-ts/README.md:374` | `BINDINGS_NAMESPACES.md:544` heading — slug matches |
| All 35 root-README relative doc links resolve | `README.md` (Quick Links, Integrations, Documentation tables) | Verified individually: QUICKSTART.md, tutorials/, examples/README.md, vantadb-ts/examples/, FAQ.md, blog/, CONTRIBUTING.md, SECURITY.md, SUPPORT.md, ADR-0030, examples/python/*.py (3), docs/api/MCP.md, EXPERIMENTAL_FEATURES.md, PROXY.md, BENCHMARKS.md, BENCHMARK_OPTIMIZATION_2026.md, ARCHITECTURE.md, MUTATION_RECOVERY_PROTOCOL.md, TEXT_INDEX_DESIGN.md, CONFIGURATION.md, MEMORY_TELEMETRY.md, PYTHON_SDK.md, PYTHON_RELEASE_POLICY.md, RELIABILITY_GATE.md, CI_POLICY.md, CHANGELOG.md, blog/*.md (3), LICENSE — **all present** |
| Node SDK quickstart is accurate | `vantadb-node/README.md:30-46` | `vantadb-node/index.d.ts:380` `export declare class VantaDb`; `:389` `static connect(path, options?)`; `put(record)`, `search(request)`, `addEdge(source,target,label,weight?)`, `graphIsDag(roots)`, `close()`; `MemorySearchHit { record, score }` (`:205`) → `hit.hits[0].record.payload` / `.score` correct |
| Node engines: "Node ≥ 18 (this package)", "vantadb-ts ≥ 22.19" | `vantadb-node/README.md:25` | `vantadb-node/package.json` `engines.node = ">=18"`; `vantadb-ts/package.json` `engines.node = ">=22.19"` |
| Provider distribution/module/class names | `providers/*/README.md` | `providers/{litellm,ollama,openai}/Cargo.toml` `name = "vantadb-<x>"`, `[lib] name = "vantadb_<x>"`; `pyproject.toml` `module-name = "vantadb_<x>"`; `.pyi` classes `VantaDBLiteLLM`/`VantaDBOllama`/`VantaDBOpenAI` with the documented ctor defaults (`namespace="<x>_store"`, `model`, `timeout`) — all match |
| Provider SDK auto-deps (`litellm>=1.0,<2`, `ollama>=0.4,<1`, `openai>=1.0,<2`) | `providers/*/README.md` | `providers/*/pyproject.toml` `dependencies = [...]` — exact match |
| `vanta-proxy` default config path + env var | `vanta-proxy/README.md:11` | `vanta-proxy/src/main.rs:29-30` → `$VANTA_PROXY_CONFIG` else `config.toml`; `vanta-proxy/config.toml` exists |
| `vantadb-server` binary name + env vars | `vantadb-server/README.md:2-4` | `vantadb-server/Cargo.toml` `[[bin]] name = "vantadb-server"`; `src/cli.rs:348,352,18` `VANTADB_PORT`/`VANTADB_HOST`/`VANTADB_STORAGE_PATH` |
| `VantaDB.create` / `Client.create` — *see S7-19* | `vantadb-ts/README.md:245` | — |
| npm `vantadb-node` not published (registry 404) | `vantadb-ts/README.md:180` | `GET https://registry.npmjs.org/vantadb-node` → 404 — the claim is **accurate** |
| npm `vantadb` published at 0.8.0 | `vantadb-ts/README.md:179` | `GET https://registry.npmjs.org/vantadb` → 200, `dist-tags.latest = 0.8.0` — accurate |
| No "coming soon" / TODO / dead sections in the root READMEs | `README.md`, `README_ES.md` | Full heading lists compared: 1:1 except the missing ES North Star (S7-16) and the missing ES Nightly badge (S7-30) |

---

## Coverage

**Files read in full (verbatim):** `README.md` (471 lines), `README_ES.md` (456 lines), `vantadb-python/README.md` (169), `vantadb-node/README.md` (77), `vantadb-ts/README.md` (397), `vantadb-wasm/README.md` (336), `vanta-proxy/README.md` (16), `vantadb-server/README.md` (5), `providers/{litellm,ollama,openai}/README.md` (53 each), `integrations/README.md` (100), `integrations/{crewai,dspy,haystack,langchain,letta,llamaindex,mem0,ollama,openai}/README.md` (62-107 each).

**Source of truth consulted for verification:**
- Packaging: `Cargo.toml` (`[package] name="vantadb"`, `[workspace.package] version="0.9.0"/rust-version="1.94.1"/license="Apache-2.0"`, `[features]`), `vantadb-python/pyproject.toml`, `vantadb-node/package.json`, `vantadb-ts/package.json`, `providers/*/{Cargo.toml,pyproject.toml}`, `rust-toolchain.toml`, `deny.toml:45-49`, `LICENSE`.
- APIs: `src/cli.rs` (full `Commands`/`ConfigCommand`/`CertificateCommand` enums), `src/bin/*.rs`, `src/cli_handlers/`, `vantadb-python/src/lib.rs`, `vantadb-python/vantadb_py/vantadb_py.pyi` (579 lines), `vantadb-python/vantadb{,_py}/__init__.py`, `vantadb-node/index.d.ts` + `dts-header.d.ts`, `vantadb-ts/src/vantadb.ts` (+ `errors.ts`, `guards.ts`), `integrations/*/vantadb_*/{__init__.py,vectorstore.py,memory.py}`, `providers/*/{src/python.rs,vantadb_*.pyi}`, `examples/python/{mem0_integration,semantic_kernel_memory,dspy_retriever}.py`, `src/sdk/` tree, `src/index/graph/types.rs`, `src/config.rs`, `src/sdk/search/debug_ops.rs`.
- Docs/claims: `docs/user/operations/BENCHMARKS.md` (numbers), `docs/api/PROXY.md`, `docs/api/BINDINGS_NAMESPACES.md`, `docs/api/NODE_SDK.md`, `docs/api/PYTHON_SDK.md`, `docs/dev/architecture/adr/ADR-{0030,0041,0047}`, `docs/user/QUICKSTART.md`, `scripts/install.sh`, `scripts/install.ps1`.
- CI: full `.github/workflows/` listing at `origin/main` (20 files) and at `origin/develop`; `ci-fleet.yml:162-180`; `release-binaries.yml:121`; `release-npm-61.yml`.
- Registries (live, one request each, read-only): npm `vantadb` / `vantadb-node` / `vantadb-wasm`; PyPI `vantadb-py` / `vantadb` / `vantadb-litellm` / `vantadb-ollama` / `vantadb-openai` / `vantadb-{crewai,dspy,haystack,langchain,letta,llamaindex,mem0}`.

**Not verified (marked NO VERIFICADO):**
- External hosts not on the allowlist: `https://mem0.ai`, `https://discord.gg/g8nqB3NtXt`, `https://www.rust-lang.org/`, `https://bundlephobia.com/package/{@orama/orama,lunr}`, `https://devpick.co/pkg/minisearch`, `https://esm.sh/vantadb`, `https://cdn.jsdelivr.net/npm/vantadb@latest/+esm` — well-formed; reachability/redirect behaviour not probed (no hammering). The jsDelivr-fails / esm.sh-works claim is **not re-tested**; it is marked "verified 2026-08-26" by the repo and is out of date by its own admission.
- GitHub Actions badge *rendered* status (I verified the workflow *files* exist or not on `origin/main`; I did not query the badge SVG, so a workflow could still be failing).
- CrewAI's own `Memory.remember(...)` / `Memory.recall(...)` API and `langchain_openai.OpenAIEmbeddings` — third-party surface, not vendored here.
- Competitive bundle sizes (23.8 KB Orama, 5.9 KB MiniSearch, 8.1 KB Lunr) — accepted as documented; not re-measured. The VantaDB row **is** internally consistent (1.77 MB raw / ~739 KB transfer appears in both `vantadb-ts/README.md:164` and `vantadb-wasm/README.md:34-37`).
- The `ci-examples.yml` / `gate-docs.yml` / `sec-codeql.yml` / `heavy-certification.yml` / `adapters-compat.yml` workflow names might exist under those names in an older release tag; I only checked `origin/main`, `origin/develop`, and the working tree.

**Method:** RBI (red-team → brainstorm → iterate). Red-team: every install command, every code sample, every badge target, every relative link and in-repo anchor, every numeric/comparative claim, and every EN↔ES pairing was re-derived from source rather than accepted from prose. Brainstorm: the alternative framing for each defect was considered (e.g. "the README may be describing a planned release" — accepted only where the repo's own text supports it, and downgraded to P3 accordingly: S7-38, S7-39). Iterate: every finding carries a concrete fix and a re-runnable verify command.

**Read-only compliance:** no file in the repo was created, edited or deleted; no git write command was run (`git status`/`ls-tree`/`rev-parse`/`branch`/`log` only). The only file written is this report, at `C:\Users\Eros\AppData\Local\Temp\opencode\audit-v090\subagents\S7.md` (outside the repo).

**SKILLS_CARGADAS:** `doubt-driven-development`, `code-review-and-quality` (SDP base set), `ponytail` (session mode: lite). Discovery run per `.opencode/references/skills-engineering.md`: lifecycle mapping = *audit / review*; `SKILLS-MANIFEST.md` grep for `readme|documentation|doc|claim|badge|link|install` surfaced `documentation-skill` (docs/ prose standard — reviewed, not applied: this is a README-accuracy audit, not a `docs/` authoring task), `writing-guidelines` (not applied: no prose-style claims in scope), `ponytail-review` (not applied: no over-engineering delta in scope), `systematic-debugging` (not applied: no fix was implemented, so no root-cause cycle was required). Total skills loaded: 2 (+ ponytail session mode) = within the ≤8 budget.
