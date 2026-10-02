---
title: WIRE-05 — Entity linking determinista + boost multi-señal en RRF (Fellegi-Sunter + embeddings)
kind: task
description: "matching multi-señal determinista (Fellegi-Sunter + embeddings, sin LLM-juez) con tests (mismos inputs → mismo score) Y boost de entidades en fuserrf opt-in, reversible y con proveniencia Y suites sdk::search/entity verdes (boost OFF..."
---

# WIRE-05 — Entity linking determinista + boost multi-señal en RRF (Fellegi-Sunter + embeddings)

> **Fase:** F2 · **Plan:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 19) · **Branch:** develop
> **Ruta:** vanta-engine · **Appetite:** max 4d · **Estado:** ⏳ IN PROGRESS
> **Commit esperado (LEAD):** `feat(entity): deterministic linking + RRF entity boost (WIRE-05)`
> **DISCOVERY:** completo (task file creado desde cero — no existía). SDP v3 ejecutado (Paso 0b).

## Contrato (verbatim — ley)

"matching multi-señal determinista (Fellegi-Sunter + embeddings, sin LLM-juez) con tests (mismos inputs → mismo score) Y boost de entidades en `fuse_rrf*` opt-in, reversible y con proveniencia Y suites `sdk::search`/`entity` verdes (boost OFF byte-idéntico al ranking actual)"

## 1. TAREA

- **`src/entity/linking.rs` (nuevo):** matching/canonicalización determinista multi-señal — Fellegi-Sunter (pesos m/u, umbrales superior/inferior) + embeddings como señal, **sin LLM** (el LLM-juez es MGR-05 paso 2/v1.0, fuera de scope). Umbral conservador: auto-link exige ≥2 señales acordando por defecto (`auto_link_bits=12 > max peso individual ≈10.95`). `mark_duplicate` manual como paso 1 (fuerza cluster, queda en proveniencia). Reversible: no muta datos — clusters son una vista.
- **Boost opt-in en `fuse_rrf*`:** contexto `EntityBoost` (mapa identidad `(namespace,key)` → cluster + weight). Grupos con ≥2 hits del mismo cluster en el set fusionado reciben `delta = weight × peers × 1/(rrf_k+1)` aditivo al score RRF, antes del sort final. Proveniencia por hit (`EntityBoostReport`: cluster, peers, `base_score`, `delta` — reversible `score − delta == base_score`). OFF (`None`/vacío) = **byte-idéntico** al ranking actual (las funciones base delegan a las boosteadas con `None`: mismo path de código).
- **Superficie:** método opt-in `Embedded::search_with_entity_boost(request, boost) -> EntityBoostedSearch { hits, boost_report }`. Rutas sin fusión (text-only/vector-only/sparse-only) no se boostean (el boost es de score RRF — documentado).
- **Acceptance = contrato verbatim** (arriba).

## 2. ARCHIVOS

**A tocar (plan Task 19):** `src/entity/mod.rs` (+`pub mod linking;`) · `src/entity/linking.rs` (nuevo) + `src/entity/linking_tests.rs` (nuevo) · `src/sdk/search/fusion.rs` (boost + tipos) · `src/sdk/search/mod.rs` (método + threading) · `src/sdk/search/hybrid.rs` (threading del boost; único caller) · `src/sdk/types/search.rs` + `src/sdk/types.rs` (re-export de tipos) · `src/sdk/search/tests.rs` (tests SDK) · `docs/api/EMBEDDED_SDK.md` (fila del método — Regla 3 + gate docs-coverage escanea `sdk/search/mod.rs`).

**NO tocar (verificado — no requiere cambios):** `src/sdk/search/explain.rs` (usa `fuse_rrf_with_report`, que queda delegando a la impl compartida → byte-idéntico sin editarlo) · `src/sdk/search/debug_ops.rs` (debug-only) · `src/planner.rs`, `src/search_profile.rs`, `src/sdk/serialization/vector_types.rs` (prohibidos WIRE-08) · `src/graph.rs` (la canonicalización se resuelve con clusters puros; no se tocan relaciones del grafo — no necesario para el contrato; sin dependencia nueva).

**PROHIBIDOS (wave/ajenos):** `src/storage/engine/insert.rs`, `src/wal.rs`, `src/config.rs`, `benches/**` (WIRE-06) · `Cargo.toml` raíz, `vantadb-ffi-core/`, bindings `*/src/lib.rs` (WIRE-07) · `perf-bench.yml` (WIP ajeno) · `src/sdk/serialization/vector_types.rs`, `src/planner.rs`, `src/search_profile.rs` (WIRE-08).

## 3. DEPENDENCIAS

- **MGR-05 🆕 PENDIENTE** (research+spec, Backlog:823) — **el slice determinista NO la requiere** (LLM-juez = paso 2/v1.0; la spec de API final de `mark_duplicate`/`auto_resolve_entities` queda en MGR-05). El diseño se fija por evidencia (F-S + umbrales citados en §9 + código real).
- **VER-08 (F5, cross-fase)** — medición del boost en harness; consume `search_with_entity_boost` (superficie pública nueva) o un bench futuro.
- **Wave F2b** · **nextTask:** WIRE-06.

## 4. REFERENCIAS + REGLAS DEL ÁREA (leídas antes de editar)

- `.opencode/rules/indexes.md` (R-1: sin números inventados) · `.opencode/rules/core-engine.md` (R-2: no exportar lo interno; R-3: sin unwrap/expect fuera de tests) · `.opencode/rules/api-contract.md` (R-6: enums públicos `#[non_exhaustive]`; R-8: lógica en core, no en bindings; R-1/R-3 docs) · `docs/dev/architecture/BOUNDARIES.md` (BND-02: `sdk/search/` NO importa `StorageEngine` directo — el boost no lo hace; BND-03 sin ciclos: `sdk→entity::linking` es acíclico, entity no importa sdk).
- Clean code Apéndice V: `src/entity/` = capa entidades (reglas puras); funciones ≤20 líneas/≤3 args (lazy: `match_score(l,r,cfg)`, `link_entities(es,cfg,manual)`, `search_with_entity_boost(req,boost)` = 2-3 args).
- Evidencia inicial (plan, verificada): `rg mark_duplicate|fellegi` = 0 hits; RRF determinista sin entidades (`fusion.rs:72-105`, tie-break `sort_hits:145-152`); 12 callers (codegraph: `fuse_rrf` 7, `fuse_rrf_many` 4, `fuse_rrf_with_report` en explain/mod).

## 5. SKILLS (SDP Paso 0b — `campaign_discover_skills_v2 phase=BUILD`)

SDP: `campaign-executor` · `progreso` · `ponytail` (base, auto) + `source-driven-development` · `doubt-driven-development` · `incremental-implementation` · `test-driven-development` · `context-engineering` (lifecycle BUILD) + `rust-write-tests` (esperado por la tarea: tests de determinismo/byte-identidad) + `api-and-interface-design` (símbolos públicos nuevos). `pinned: []` (SDP v3 sin pins).
**SKILLS_CARGADAS:** `source-driven-development`, `test-driven-development`, `incremental-implementation`, `doubt-driven-development`, `rust-write-tests`, `api-and-interface-design`.

## 6. HERRAMIENTAS

- `codegraph_codegraph_explore` / codebase-memory-mcp **antes** de grep (hecho: blast radius 12 callers, call flow `search_impl → fuse_rrf_with_report → fuse_rrf`).
- Iteración: `CARGO_BUILD_JOBS=2 cargo nextest run --profile audit -p vantadb -E 'test(entity) or test(fusion) or test(hybrid)'` (regla dura `-p`; presión de memoria → 2 jobs).
- `cargo clippy -p vantadb --all-targets -- -D warnings` · `cargo fmt --check` · `pwsh scripts/validate-docs-coverage.ps1` (gate: `sdk/search/mod.rs:pub fn` nuevos deben aparecer en `EMBEDDED_SDK.md`).
- `campaign_verify_cmd` por step (forma estable `pwsh -NoProfile scripts/...`; retry si -1 — flake FIND-173).

## 7. INVESTIGACIÓN CÓDIGO (blast radius — codegraph + trace_path)

| Símbolo | Callers (inbound) | Efecto |
|---|---|---|
| `fuse_rrf` (fusion.rs:72) | 7: `fusion.rs` (tests+`fuse_rrf_with_report`), `hybrid.rs:42`, `debug_ops.rs:269`, `explain.rs` (vía `with_report`) | Pasa a delegar en impl compartida con `None` → byte-idéntico |
| `fuse_rrf_many` (fusion.rs:97) | 4: `hybrid.rs:40`, `debug_ops.rs:264,295,341`, `explain.rs:79,106,149`, `mod.rs:179,208,239,312,342` | Ídem — delegación |
| `fuse_rrf_with_report` (:106) | `mod.rs:186`, `explain.rs:88` | Delegación interna (devuelve solo los 2 primeros del tuple) |
| `hybrid_search` (hybrid.rs:7) | 1: `mod.rs:288` | + parámetro `Option<&EntityBoost>` + report de retorno (crate-internal) |
| `search_impl` (mod.rs:100) | 2: `search:72`, `search_with_method:91` | + threading boost; retorna `(hits, report)`; los 2 callers descartan report |
| `SearchExplanationHit` / `MemorySearchHit` | NO se modifican (vector_types.rs prohibido) — proveniencia va en sidecar `EntityBoostReport` | Cero impacto serialización/snapshot public-api |

**Veredicto:** cambio aditivo. 3 funciones base mantienen firma y semántica (delegan); superficie nueva = linking (módulo público entidad) + boost (tipos + método SDK). Blast radius del *comportamiento*: 0 rutas existentes cambian (OFF byte-idéntico verificado por test). Riesgo: 🟢 bajo con las mitigaciones del contrato.

## 8. INVESTIGACIÓN PROBLEMA (Fellegi-Sunter determinista)

- **Formulación** (citada §9): weight total `W = prior + Σ_i log2(m_i/u_i)` para señales en acuerdo; desacuerdo `log2((1−m_i)/(1−u_i))`; posterior `2^W/(1+2^W)`; decisión por umbral superior (auto) / inferior → banda de revisión manual (F-S 1969).
- **Defaults conservadores** (documentados, overrideables por `SignalWeight::new(m,u)`):
  `Name m=.95/u=.001` · `Email m=.99/u=.0005` · `Phone m=.95/u=.0005` · `Embedding m=.90/u=.05` (agree: coseno ≥ `embedding_threshold=.90`). `auto_link_bits=12` (posterior ≈0.99976; > max señal individual ⇒ exige ≥2 señales acordando) · `review_bits=4` · `prior_bits=0` (pares ya candidatos; EM queda fuera de scope).
- **Determinismo:** comparación por kind en orden fijo (enum order), "primer signal del kind gana", sins `HashMap` en output, floats sin reordenamiento (mismas ops → mismo bit pattern), redondeo de evidencia `{:.4}`.
- **Emulación EM/UF:** unión determinista por raíz lex-menor; clusters/decisiones ordenados (canonical, left,right).
- **Stop conditions (plan):** (a) si exigiera LLM en camino crítico → recortar a F-S+embeddings ✅ (no lo exige); (b) si el boost no puede ser byte-idéntico con OFF → rediseñar ANTES de tocar `fuse_rrf` ✅ (delegación con `None` = mismo path; test lo prueba).
- **Pre-mortem:** F1 matching agresivo → auto-link multi-señal + `mark_duplicate` manual paso 1 + test de umbral; F2 boost no reversible → proveniencia con `base_score`/`delta` + test OFF byte-idéntico + test `score−delta==base_score`; F3 dep MGR-05 ausente → defaults por evidencia citada + overrides (esta sección).

## 9. INVESTIGACIÓN INTERNET (validación + citas)

1. **Splink (UK Ministry of Justice)** — *The Fellegi-Sunter Model*: `M = log2(λ/(1−λ)) + Σ log2(m_i/u_i)` (Bayes factors aditivos), `Pr(Match|Obs)=2^M/(1+2^M)`; umbrales de ejemplo: peso 7 → prob 0.99, 4 → 0.95.
   Fuente: https://moj-analytical-services.github.io/splink/topic_guides/theory/fellegi_sunter.html (fetch ✅ 2026-09-27).
2. **Wikipedia — Record linkage** (basado en Fellegi & Sunter 1969, JASA 64(328):1183-1210): likelihood ratio de acuerdo `m/u`, de desacuerdo `(1−m)/(1−u)`; umbral superior/inferior con banda de revisión clerical; independencia condicional ⇒ pesos aditivos; blocking para candidatos.
   Fuente: https://en.wikipedia.org/wiki/Record_linkage (fetch ✅ 2026-09-27).
- El umbral de embeddings (0.90) queda como **default conservador configurable** (sin fuente canónica específica — marcado como tal; el valor definitivo es dato-dependiente y se mide en VER-08).

## 10. VALIDACIÓN + CIERRE

- Verify contrato: `cargo nextest run --profile audit -p vantadb -E 'test(entity) or test(fusion) or test(hybrid)'` (incluye test OFF byte-idéntico) + clippy `-D warnings` + fmt.
- Verify full área: `--workspace --build-jobs 2` en el cierre + `validate-docs-coverage.ps1`.
- DoD multi-nivel (§ abajo). **NO commit (LEAD)** · **NO self-review (LEAD)** — el review P2-01 adversarial del tier `src/sdk/**` lo hace el lead con agente distinto. RESULTADO §7 obligatorio.

## Spec (SDD — feature-add: símbolos públicos nuevos)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Superficie del boost | (A) fusion-only crate-visible (menor diff, sin opt-in de usuario, riesgo dead_code) / (B) + método `Embedded::search_with_entity_boost` (+fila docs, +review sdk) | **B** — "opt-in" sin superficie no existe y VER-08 necesita path público | ✅ decidido-por-evidencia (contrato "opt-in"; plan DoD VER-08; `validate-docs-coverage.ps1:64` escanea sdk/search/mod.rs) |
| 2 | Mutar `MemorySearchHit`/`MemorySearchRequest` para config del boost | (A) campos nuevos (prohibido: `vector_types.rs` = WIRE-08) / (B) sidecar `EntityBoostReport` + param explícito | **B** | ✅ decidido-por-restricción (prohibidos plan Task 19) |
| 3 | Semántica del boost | (A) apoyo de co-ocurrencia: hits de un cluster con ≥2 miembros en el set fusionado suben por peer / (B) booleano "es entidad" / (C) match contra query | **A** — determinista, medible, sin NER/consulta; B/C necesitan señales fuera de scope | ✅ decidido-por-evidencia (contrato determinista; MGR-05 "fusión reversible con proveniencia") |
| 4 | Formulación F-S | (A) clásica bipartita agree/disagree con `log2(m/u)` / `(1-m)/(1-u)` / (B) multi-nivel tipo Splink | **A** — 2 niveles bastan para señales normalizadas; B requiere niveles por señal (YAGNI) | ✅ decidido-por-evidencia (Splink §Match Weights; Wikipedia §Probabilistic) |
| 5 | Umbral auto-link | (A) 7 bits (0.99) / (B) 12 bits (≥2 señales) / (C) EM-estimado | **B** — "conservador" = multi-señal; EM fuera de scope F2 | ✅ decidido-por-evidencia (Splink: 7→0.99; max señal individual 10.95 < 12) |
| 6 | Módulo del boost | (A) `fusion.rs` (junto a RRF) / (B) módulo nuevo `sdk/search/entity_boost.rs` | **A** — todo el contrato habla de `fuse_rrf*`; un archivo menos | ✅ decidido-por-evidencia (plan: "boost opt-in en `fuse_rrf*`" fusion.rs) |
| 7 | Dependencia `sdk → entity::linking` | (A) reusar `EntityLinkIndex` de linking en fusion (edge sdk→entity) / (B) tipos de boost autocontenidos en fusion + linking independiente | **B** — cero edges nuevos; linking produce clusters, el caller arma el mapa | ✅ decidido-por-restricción (BND-03; minimizar acoplamiento) |
| 8 | Enums públicos nuevos | `#[non_exhaustive]` | **Sí** (SignalKind/SignalOutcome/LinkVerdict) | ✅ decidido-por-regla (`api-contract.md` R-6) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/entity/mod.rs` · `src/entity/checker.rs` · `src/entity/scene.rs` · `src/sdk/search/fusion.rs` · `src/sdk/search/hybrid.rs` · `src/sdk/search/explain.rs` · `src/sdk/search/mod.rs` · `src/sdk/search/debug.rs` · `src/sdk/search/debug_ops.rs` (uso fusion) · `src/sdk/types.rs` (1-36) · `src/sdk/types/search.rs` (1-150) · `src/sdk/serialization/vector_types.rs` (1-140) · `src/node/field.rs` (FieldValue) · `.opencode/rules/{indexes,core-engine,api-contract}.md` · `BOUNDARIES.md` (30-109) · `scripts/validate-docs-coverage.ps1` (40-119) · `docs/api/EMBEDDED_SDK.md` (70-99) · `docs/dev/tasks/WIRE-04.md` (formato) · `.opencode/references/clean-code-clean-architecture.md` Apéndice V.
- **Referencias hacia dentro (imports de lo que toco):** `fusion` ← `mod.rs`, `hybrid.rs`, `explain.rs`, `debug_ops.rs`; `entity` ← `cli_server_auth_tests.rs`; `sdk/types` ← bindings/serialización (los tipos nuevos son additivos).
- **Referencias entrantes (grep):** `fuse_rrf(` 12 call-sites (7+4+report); `search_impl` 2; `hybrid_search` (sdk) 1; `EntityStore` usado en tests/auth — `linking` no lo toca.
- **Veredicto:** 🟢 aditivo — firmas base intactas (delegación), 0 cambios de serialización, 0 migración de datos, snapshots public-api potablemente crecen solo con símbolos nuevos (verificar `tests/api/public-api.txt` al final).

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) RRF OFF ⟹ ranking byte-idéntico (`fuse_rrf*` sin boost = mismo Vec que hoy); (2) determinismo total del matching y del boost (mismos inputs → mismos scores/orden); (3) `link_entities` NO muta datos almacenados (vista/clusters); (4) sin LLM en camino crítico; (5) no romper suites `entity`/`sdk::search`; (6) `sdk/search` no importa `StorageEngine` (BND-02).
- **Comandos de verificación:** `CARGO_BUILD_JOBS=2 cargo nextest run --profile audit -p vantadb -E 'test(entity) or test(fusion) or test(hybrid)'` → todos verdes; `cargo clippy -p vantadb --all-targets -- -D warnings`; `pwsh scripts/validate-docs-coverage.ps1` → 0 gaps.
- **Deuda pendiente:** EM-estimación de m/u, top-K vector-first para candidatos (hoy O(n²) documentado `ponytail:`), LLM-juez (MGR-05 paso 2), medición del boost (VER-08), superficie explain boosteada (`explain_memory_search` sigue sin boost — candidato WIRE-08).

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda nueva neta — el único techo conocido (matching O(n²) sin blocking/top-K) queda marcado con `ponytail:` y con upgrade path; no se introduce `unsafe`, ni unwrap, ni alloc en hot path nuevo del search (el boost corre sobre candidatos ya materializados pre-sort). Pago colateral: se elimina duplicación de fusión (implementación única compartida por `fuse_rrf`/`fuse_rrf_many`/`with_report`).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable §Contrato se cumple + fmt/clippy/nextest del área + tests del cambio pasan |
| **Commit** | Commit atómico conventional (LEAD): `feat(entity): deterministic linking + RRF entity boost (WIRE-05)` — `git diff` limpio, verificación mecánica |
| **Release** | No aplica a este slice (sin bump/publish; lo cubre el cierre de wave por LEAD) |

## Herramientas necesarias

- codegraph/CBM (blast radius ✅) · cargo nextest/clippy/fmt (regla `-p`) · `campaign_verify_cmd` · docs-coverage script.

## Investigation Notes

- Ver §8/§9. Extra: `search_profile.rs` define `RRF_K=60` (top contribution 1/61≈0.0164) — el weight del boost es *fracción* de la contribución top (`weight × 1/(rrf_k+1)`) para ser invariante al `rrf_k` del profile; default 0.25.
- `debug.rs` usa convención de identidad `"{namespace}\0{key}"` (`hit_identities`) — la proveniencia usa la misma (peers).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — resueltas en §7-§9 (contrato, formulación F-S, umbrales, superficie) |
| Pendientes de ejecución (downhill) | 0 worker — cierre = review adversarial + commit (LEAD, por instrucción) |
| % completado | 100% worker (implementación + verify; pendiente LEAD) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY —** toca trust boundary de input (embeddings/ids de usuario): validación en boundary (`SignalWeight::new` validado, ids no vacíos/únicos, config → `Result`), cero `unwrap`, cero panic paths (zero-norm embedding → Missing, no división por cero). Sin red/auth/FFI/deps nuevas. No requiere `security-and-hardening` completo; aplicado su principio de validación en frontera.
- [x] **PERFORMANCE —** hot path tocado: `fuse_rrf*` (search). Sin regresión OFF (mismo path; test byte-idéntico); ON = O(H) sobre candidatos ya fusionados + O(k) provenance (sin allocs en el camino OFF). No se toca `benches/**` (WIRE-06); medición del boost = VER-08 (F5). Baseline canónico no requerido por el contrato (no hay claim de performance — Regla 9/11).

## Steps

### Step 1: RED linking — tests de determinismo F-S
- **Archivos:** `src/entity/linking_tests.rs` (nuevo) + `src/entity/linking.rs` (stubs mínimos) + `src/entity/mod.rs`
- **Acción:** tests RED: determinismo (mismos inputs→mismo score), normalización texto, umbral conservador multi-señal, embedding threshold, missing neutral, manual `mark_duplicate`, clusters ordenados, overrides m/u, posterior Splink (W=9.48→0.999), errores de boundary.
- **Verify:** `CARGO_BUILD_JOBS=2 cargo nextest run --profile audit -p vantadb --lib -E 'test(linking)'` → RED: 15 tests, 0 passed (stubs), fallos por razón correcta
- **Estado:** ✅

### Step 2: GREEN linking — implementación determinista
- **Archivos:** `src/entity/linking.rs`
- **Acción:** F-S completo (SignalWeight validado, match_score, link_entities con unión determinista, LinkReport clusters/decisiones, `signals_from_fields`).
- **Verify:** mismo filtro → **15/15 ✅** (0.27s)
- **Estado:** ✅

### Step 3: RED boost — tests byte-identidad + promoción + proveniencia
- **Archivos:** `src/sdk/search/fusion.rs` (mod tests, 8 tests nuevos)
- **Acción:** tests RED: OFF `None` byte-idéntico; ON promueve peers de cluster; proveniencia reversible; determinismo doble corrida; vacío/non-finite = no-op; singleton sin boost; `fuse_rrf_many` ídem.
- **Verify:** RED documentado: E0425/E0433 `EntityBoost`/`fuse_rrf_with_entity_boost` not found
- **Estado:** ✅

### Step 4: GREEN boost — impl compartida + delegación base
- **Archivos:** `src/sdk/search/fusion.rs`
- **Acción:** `EntityBoost`/`EntityBoostProvenance`/`EntityBoostReport`/`EntityBoostedSearch`; `apply_entity_boost`; impls compartidas; `fuse_rrf`/`fuse_rrf_many`/`fuse_rrf_with_report` delegan con `None` (firma intacta → byte-idéntico por construcción).
- **Verify:** `-E 'test(fusion)'` → **36/36 ✅**
- **Estado:** ✅

### Step 5: Opt-in SDK — método + threading + tests SDK
- **Archivos:** `src/sdk/search/mod.rs` · `src/sdk/search/hybrid.rs` · `src/sdk/types/search.rs` · `src/sdk/types.rs` · `src/sdk/mod.rs` · `src/sdk/search/tests.rs`
- **Acción:** `EntityBoostedSearch`; `Embedded::search_with_entity_boost`; threading en `search_impl`/`hybrid_search` (todas las rutas de fusión, incl. explain); tests SDK (OFF idéntico a `search`, ON promueve + provenance, determinismo, explain).
- **Verify:** `-E 'test(entity) or test(fusion) or test(hybrid)'` → **112/112 ✅**
- **Estado:** ✅

### Step 6: Docs + verify full del área
- **Archivos:** `docs/api/EMBEDDED_SDK.md`
- **Acción:** fila de `search_with_entity_boost` + sección "Entity Linking + Entity Boost" (fórmula F-S, defaults, ejemplo). Verify: fmt/clippy/nextest área + suite completa `--lib` (2160/2160) + docs-coverage (0 gaps).
- **Estado:** ✅

## Verificación (evidencia)

| Check | Comando | Resultado |
|---|---|---|
| linking (RED→GREEN) | `cargo nextest run --profile audit -p vantadb --lib -E 'test(linking)'` | RED 0/15 → **GREEN 15/15** |
| fusion boost | `... -E 'test(fusion)'` | **36/36** |
| área (contrato) | `... -E 'test(entity) or test(fusion) or test(hybrid)'` | **112/112** (dos corridas) |
| suite completa lib | `CARGO_BUILD_JOBS=2 cargo nextest run --profile audit -p vantadb --lib` | **2160 passed / 0 failed** (2 skipped pre-existentes; 1 SLOW pre-existente HNSW) |
| fmt | `cargo fmt --all -- --check` | ✅ mis archivos — único diff: `src/ingestion.rs` (WIP ajeno, no tocado) |
| clippy | `cargo clippy -p vantadb --all-targets -- -D warnings -A dead_code` | ✅ Finish (el `-A dead_code` neutraliza el ÚNICO warning ajeno: `has_active_transaction` `src/storage/engine/mod.rs:601` — añadido sin commitear por wave WIRE-06/07 en vuelo; fuera de mi scope prohibido) |
| docs coverage | `pwsh -NoProfile scripts/validate-docs-coverage.ps1` | ✅ **0 gaps** (29 items sdk ok, incluye `search_with_entity_boost`) |

**Contrato — evidencia por cláusula:**
1. *"matching multi-señal determinista (F-S + embeddings, sin LLM-juez) con tests (mismos inputs → mismo score)"*: `entity::linking` — `linking_match_score_is_deterministic` (assert_eq bit a bit), `linking_clusters_are_deterministic_and_sorted`, `linking_posterior_matches_splink_reference`; sin LLM en camino crítico ✅
2. *"boost de entidades en `fuse_rrf*` opt-in, reversible y con proveniencia"*: `fuse_rrf_with_entity_boost`/`fuse_rrf_many_with_entity_boost` + `EntityBoost`/`EntityBoostReport` + `Embedded::search_with_entity_boost` — `fuse_rrf_with_entity_boost_promotes_cluster_peers` (peers, cluster, `score−delta==base_score`), `..._empty_index_is_noop`, `..._non_finite_weight_is_noop`, `..._singleton_cluster_is_noop` ✅
3. *"suites `sdk::search`/`entity` verdes (boost OFF byte-idéntico al ranking actual)"*: `fuse_rrf_with_entity_boost_off_is_byte_identical` + `fuse_rrf_many_..._off_...` + `test_search_hybrid_entity_boost_off_is_identical_to_search` (`assert_eq!` del Vec completo) + 112/112 área + 2160/2160 full lib ✅

## Context Save Point

- **Última acción (2026-09-28):** steps 1–6 ✅. Implementado `entity::linking` (F-S determinista + `mark_duplicate` manual + clusters), boost opt-in reversible con proveniencia en `fuse_rrf*`, método SDK `search_with_entity_boost`, docs. Ver §Verificación.
- **Pendiente (LEAD, NO worker):** review P2-01 **adversarial** (diff toca `src/sdk/**` + superficie pública nueva) por agente distinto + commit local `feat(entity): deterministic linking + RRF entity boost (WIRE-05)` + sync del plan file. SIN self-review (instrucción explícita).
- **Condiciones externas observadas:** (a) `cargo nextest` sin `--lib` falla por `vanta-cli.exe` en uso (FIND-177, sesiones MCP vivas) — usar `--lib`; (b) working tree compartido: `src/config.rs`/`src/ingestion.rs`/`src/storage/engine/mod.rs` con WIP de WIRE-06/07 en vuelo → `-D warnings` de clippy y `fmt --check` global afectados por ESA deuda, no por WIRE-05 (evidencia: `git diff HEAD`); (c) race de compilación con la sesión paralela: un run falló por `config.rs` a mitad de edición y pasó al reintentar (no es falla propia).
- **Decisiones de diseño:** ver §Spec (A boost por co-ocurrencia de cluster; tipos autocontenidos en fusion sin edge sdk→entity; delegación base→boosted con `None` para garantizar byte-identidad; delta relativo a `1/(rrf_k+1)`).

## Review (GATE — agente distinto, P2-01)

> Tier del diff: **adversarial** (`src/sdk/**` + superficie pública nueva). NO lo ejecuta este implementador — lo delega el LEAD con agente distinto (`vanta-review`/`vanta-audit`) antes del commit. Payload HARD-07 en la recitation de cierre del LEAD.

- **Revisor:** (LEAD) vanta-review/vanta-audit — **pendiente**
- **Enfoque:** semántica del boost (promoción sensata y reversible), umbrales F-S conservadores, boundary BND-02/03, superficie pública mínima, naming.
- **Cómo se probó:** §Verificación (mecánica, sin auto-reporte).
- **Veredicto:** ⬜ pendiente (LEAD)

## Dependencias

- MGR-05 🆕 (spec API final — NO bloquea slice determinista) · VER-08 (medición, F5) · nextTask: WIRE-06.

## Notas

- **Gate D (question-gates):** disparadores evaluados — símbolos públicos nuevos ✔ y hot path ✔ — **resueltos por el plan** (Task 19 Gate Result ✅ DO + contrato explícito + instrucción del orquestador "NO self-review/NO commit (LEAD)"): no se re-pregunta, se ejecuta el contrato y se entrega el RESULTADO para el gate del lead.
- Boost = solo rutas RRF (documentado); text-only/vector-only no se boostean (no hay score RRF que boostear).
- `explain_memory_search` público queda sin boost (deuda anotada); `search_with_entity_boost` con `request.explain=true` SÍ adhiere explicaciones por hit + report.
