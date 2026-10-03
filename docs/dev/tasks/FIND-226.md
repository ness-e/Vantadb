---
title: "TASK FIND-226: ASan — símbolos + triage de los leaks (1.81 GB)"
kind: task
description: "Triage cerrado: 99.96% de los bytes = 28 engines in-memory filtrados por Box::leak innecesario en tests de graph/gds (64 MiB c/u); fix aplicado + llvm-symbolizer en el job ASan; residual ~700 KB a confirmar post-push"
---

# TASK FIND-226: ASan — símbolos + triage de los leaks (1.81 GB)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-02-post-release-0.8.0.md` (Task 9, Wave 1)
- **Fuente:** `docs/dev/Backlog.md` (FIND-226, L419)
- **Esfuerzo:** 🔴 1-2d | **Appetite:** 3d
- **Prioridad:** 🟡 Media
- **Tipo:** Mixto (CI/CD + tests Rust) — blast radius: 1 job best-effort + tests unitarios; cero código de producción
- **Turns estimados:** 30-60
- **Creado:** 2026-10-03T04:10Z | **last-synced:** 2026-10-03T04:10Z
- **Estado:** ⏳ IN PROGRESS
- **Campaign ID:** post-release-0.8.0-20261002
- **Incógnitas (uphill):** 1 → 0 (`alcance real de los leaks` resuelto con evidencia de logs + mapeo a código: test-exit deliberado, 99.96% de bytes; residual <1 KB-por-allocs se confirma post-push)
- **Pendientes (downhill):** 1 (Step 7 post-push, orquestador/owner); Steps 1-6 ✅

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `.github/workflows/ci-rust.yml` job `sanitizer-asan` (best-effort, `continue-on-error`) — único consumidor del job. `GraphTraverser`/`GraphDataScience` (src/graph.rs, src/gds.rs) — usados por tests unitarios + CLI (`executor`/`gds` consumers). `.lsan_suppressions` (raíz) — consumido por `LSAN_OPTIONS` del job (archivo listado en el prompt del orquestador para FIND-226). |
| Callees | `File::create_in_memory` → `AlignedBytes::zeroed` (src/storage/vfile.rs:152-176, src/storage/vfile_mmap.rs:461-539); `StorageEngine::open_with_config` rama InMemory (src/storage/engine/init.rs:38-44); `llvm-symbolizer` del paquete apt `llvm`. |
| Implicaciones | Contrato público de `GraphTraverser<'a>::new`/`GraphDataScience<'a>::new` **NO cambia** (ya toman `&'a StorageEngine`). Cambio de comportamiento solo en tests (dejan de filtrar memoria). Job ASan: +1 paquete apt + 1 step de resolución; el reporte pasa de direcciones a símbolos. Cero impacto en producción, wire format, performance o datos. |

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `.github/workflows/ci-rust.yml` (jobs `sanitizer-asan` L630-664 y `sanitizer-tsan` L666-701 — este último NO se toca), `.lsan_suppressions` (2L), `src/lsm.rs` (323L), `src/storage/vfile.rs` (914L), `src/storage/vfile_mmap.rs` (626L), `src/storage/engine/init.rs` (L1-140), `src/graph.rs` (parcial: cabecera + `mod tests`), `src/gds.rs` (parcial: cabecera + `mod tests`), `docs/dev/operations/CI_POLICY.md` (§1 Fast Gate, §sanitizers), `.opencode/rules/release-ci.md` (42L), `.opencode/rules/durability.md` (34L).
- **Referencias entrantes (grep por nombre):** `Box::leak` = 28 sitios, todos dentro de `#[cfg(test)] mod tests` (graph.rs 21, gds.rs 7). `GraphTraverser::new`/`GraphDataScience::new` = construcciones en tests + usos internos de `gds.rs` (`GraphTraverser::new(self.storage)` L45). `sanitizer-asan` referenciado por CI_POLICY.md §1 (inventario de workflows) — sin cambios de política.
- **Referencias salientes de los editados:** `ci-rust.yml` → actions pinneadas por SHA + apt packages; `graph.rs`/`gds.rs` → `StorageEngine`, `UnifiedNode`, `Edge`, `tempfile` (tests).
- **Veredicto impacto:** **BAJO / aislado.** El fix es test-only (28 líneas mecánicas) y no altera ninguna firma pública; el job es best-effort (no gatea releases, `continue-on-error` con `# CATEGORY: BEST-EFFORT` ya presente). Nada se rompe si se revierte: el diff es 100% reversible.

## Evidencia dura (runs ASan — Regla 9/11)

### Run actual (main, post-0.8.0) — job 111086294644 (run 37082038794, 2026-10-03T00:34Z)

```
SUMMARY: AddressSanitizer: 1745658558 byte(s) leaked in 1358 allocation(s).
1358 allocations (objetos) = 28 objetos Direct (2728 B c/u) + 1330 objetos Indirect en 1025 bloques
  = 26 bloques × 67108864 B (64 MiB) + 999 bloques chicas (1304 objetos, 751,710 B).
26 × 64 MiB = 1,744,830,464 B = 99.95% de los bytes.
```

- **Sin símbolos:** cada frame del reporte es solo `#N 0xADDR (…/vantadb-hash+0xOFFSET) (BuildId: …)` — ningún nombre de función, sin `llvm-symbolizer` en el runner. `grep -i symbolizer` en el log = 0 menciones. Sin símbolos no hay triage (ni siquiera las `LSAN_OPTIONS=suppressions` pueden matchear: el formato `leak:<pattern>` matchea contra el stack **simbolizado**, doc oficial LeakSanitizer §Suppressions).
- **Firma de los stacks:** las 26 pilas de 64 MiB comparten frames idénticos de asignación (`0x561fb2cef722 → … → 0x561fb38e5cca` + frame variable por test); las 28 Direct 2728 B comparten `0x561fb2cef544 → 0x561fb3297d74 → 0x561fb389c***` (frame variable por test). Cluster determinístico por sitio de llamada.

### Pre-existente (main 2026-09-25) — job 107968833319 (run 36101773949)

```
SUMMARY: AddressSanitizer: 1879861353 byte(s) leaked in 1372 allocation(s).
28 × Direct 2648 B + 28 × Indirect 64 MiB.
```

- Mismo patrón 8 días antes, **antes** del ciclo 0.8.0. El tamaño del Direct cambió 2648 → 2728 B (el struct `StorageEngine` creció ~80 B en el ciclo) — corroboración de que el Direct ES el box del engine. El conteo de 64 MiB indirectos varía entre runs (26 hoy, 28 el 09-25); causa no determinada (posible estado de reachability en el leak-check) — irrelevante para el fix: las 28 raíces Direct son las mismas en ambos runs y los buffers cuelgan de ellas.

### Mapeo a código (¿qué asigna exactamente 64 MiB en heap?)

- **67108864 B = 64 MiB exactos.** Los únicos sitios que asignan ese tamaño en heap son `File::create_in_memory(64 * MIB)`:
  - `src/storage/engine/init.rs:40` — rama `BackendKind::InMemory` de `open_with_config`: `let vs = File::create_in_memory(64 * MIB);`
  - `src/storage/vfile.rs:152-162` — `AlignedBytes::zeroed(size)` con `size = initial_size.max(STORAGE_ALIGNMENT)` = 64 MiB (heap, `alloc_zeroed`, align 4).
  - La ruta persistente (`src/lsm.rs:156`: `File::open(path, 64 * 1024 * 1024)`) usa **mmap** (memmap2) o el shim — mmap NO es heap y LSan no lo reporta. El reporte es 100% consistente con engines **in-memory**.
- **28 Direct = 28 sitios `Box::leak(Box::new(storage))`** — todos los tests usan `setup_storage()` con `Config { backend_kind: BackendKind::InMemory }` (graph.rs:564-573, gds.rs:193-202) y filtran el engine a propósito:
  - `src/graph.rs`: 21 sitios (tests `test_bfs_*`, `test_dfs_*`, `test_topological_sort_*`, `test_is_dag_*`) — L616-903.
  - `src/gds.rs`: 7 sitios (tests `test_page_rank_*`, `test_centrality_*`) — L229-394.
- **El `Box::leak` es innecesario** (causa raíz): ambos constructores ya toman un préstamo normal — `GraphTraverser<'a>::new(storage: &'a StorageEngine)` (graph.rs:56-60) y `GraphDataScience<'a>::new(storage: &'a StorageEngine)` (gds.rs:22-26). `storage` vive todo el test; el leak no compra nada.
- **Clasificación (contrato 2):**
  - **Test-exit / estructural (deliberado):** 99.96% de los bytes — engines in-memory filtrados por `Box::leak` en tests unitarios. No es leak de diseño del motor; en producción `StorageEngine` se dropea normalmente (no existe `Box::leak` ni `mem::forget` fuera de `mod tests` — verificado por grep).
  - **Residual (<0.04%, ~751 KB en 999 allocs):** indirectos alcanzables desde los mismos engines filtrados (paths, caches, backend) — se espera que desaparezcan con el fix; los que persistan se triarán en el próximo reporte simbolizado (Step post-push).
  - **FIND-213 (NL_POOL):** no aparece en las firmas top (no hay frame de 64 MiB ni 2728 B con ese patrón); queda como candidato a verificar en el reporte simbolizado — no se absorbe por ahora.

## Decisión (contratos 2 y 3)

1. **Símbolos (contrato 1):** instalar `llvm` (apt, provee `/usr/bin/llvm-symbolizer` — filelist oficial Ubuntu noble) en el job + step de resolución que exporta `ASAN_SYMBOLIZER_PATH` (mecanismo documentado en clang.llvm.org/docs/AddressSanitizer.html §"Symbolizing the Reports"). Alternativa descartada: componente rustup `llvm-tools` — **no incluye** `llvm-symbolizer` (verificado en el manifest del componente instalado: solo `llvm-cov`/`llvm-profdata`). No se necesita `-Cdebuginfo` (build dev = debuginfo por defecto; la carencia era el binario, no el debug info). El step **falla fuerte** si el binario no aparece (un reporte sin símbolos es el no-op que este task elimina; el job es best-effort y no gatea nada).
2. **Leaks (contratos 2 y 3):** **fix, sin supresión LSAN**. Se eliminan los 28 `Box::leak` innecesarios (graph.rs, gds.rs) — 28 líneas mecánicas test-only. Razones contra la supresión: (a) el leak es nuestro y es trivial de arreglar; (b) una supresión amplia (p.ej. `leak:File::create_in_memory` o `leak:*graph::tests*`) enmascararía leaks reales futuros del mismo sitio; (c) las supresiones requieren stacks simbolizados — recién con el symbolizer serían efectivas, justo cuando queremos la señal limpia. `.lsan_suppressions` se mantiene con su entrada `leak:rocksdb` (RocksDB opt-in) + nota de esta decisión.
3. **No deriva a fix de motor:** no hay cambio de diseño pendiente (los constructores ya aceptan `&'a`; ningún símbolo público nuevo — Gate D/spec: no aplica).

## Contrato (del plan)

1. El job ASan corre con `llvm-symbolizer` disponible → el próximo reporte tiene símbolos. **Verificable post-push** (ASan no corre en Windows local): `gh run view <job-id> --log | rg "llvm-symbolizer|Using symbolizer"` + frames con nombres de función.
2. Clasificación documentada: test-exit/estructural vs leaks reales. **✅ este archivo §Evidencia dura + §Decisión.**
3. Leaks reales → fix o supresión LSAN documentada; decisión registrada en FIND-226. **✅ fix aplicado (§Decisión 2); residual a confirmar post-push.**

## Steps

- [x] **Step 1 — Evidencia dura de runs:** job 111086294644 (main 10-03) + job 107968833319 (main 09-25); SUMMARY + clustering por clase (28 Direct / 26-28×64 MiB / 999 chicas); confirmación de stacks sin símbolos y pre-existencia. ✅
- [x] **Step 2 — Mapping a código:** 64 MiB = `File::create_in_memory(64*MIB)` (init.rs:40 → vfile.rs:152) en engines InMemory; 28 = `Box::leak` en tests graph(21)/gds(7); constructores ya toman `&'a` (causa raíz: leak innecesario). ✅
- [x] **Step 3 — Job fix:** `ci-rust.yml` job `sanitizer-asan`: `llvm` en apt + step "Resolve llvm-symbolizer" → `ASAN_SYMBOLIZER_PATH` (GITHUB_ENV, con fallback `find /usr/lib/llvm-*`, fail-loud si falta). NO se tocó `sanitizer-tsan`. **Verify:** `actionlint .github/workflows/ci-rust.yml` → exit 0 ✅
- [x] **Step 4 — Leak fix:** `Box::leak(Box::new(storage))` → `&storage` en `src/graph.rs` (21) + `src/gds.rs` (7) = 28 reemplazos; asserts intactos. **Verify:** `cargo fmt --check` exit 0 ✅ · `cargo clippy -p vantadb --all-targets -- -D warnings` exit 0 (222s) ✅ · `cargo nextest run --profile audit -p vantadb -E "test(/graph::tests/) | test(/gds::tests/)"` → **55/55 passed** ✅ (el filtro unanchored también captura otros módulos `*::graph::tests` — index/sdk — además de los 28 de graph/gds; 1er intento falló por estado transitorio del target compartido — "metadata stub for std" durante una compilación de dev-deps a medio construir; resuelto al completar el build, re-run limpio sin cambios de código)
- [x] **Step 5 — Docs:** `.lsan_suppressions` nota FIND-226 (decisión: fix, no supresión) + este task file. **Verify:** `node scripts/docs/check-links.mjs` exit 0 ✅ · `node scripts/docs/check-docs.mjs` exit 0 ✅ (FIND-226.md listado como "orphan" en reporting-only, igual que el resto de tasks activas) ✅
- [x] **Step 6 — Cierre local:** OCR delegation (grupos 1-3 aplicados a MIS archivos: 0 Critical/High) + review P2-01 + commit LOCAL conventional. ✅ (ver §Review para veredicto)
- [ ] **Step 7 — Post-push (orquestador/owner):** push develop → próximo run ASan: (a) `gh run list --workflow=ci-rust.yml --limit 1`; (b) `gh run view --job <asan-job> --log | rg "Using symbolizer|SUMMARY: AddressSanitizer"` → esperado: símbolos presentes (frames `vantadb::…`) y bytes ≪ 1.8 GB (idealmente ~0; residual máximo esperado <1 MB). Criterio: símbolos = contrato 1; sin las clases engine×64 MiB = contrato 3 confirmado. Si queda residual >1 MB, triar con el reporte simbolizado (FIND nuevo si es ajeno). ⬜ PENDING orquestador.

## Review (P2-01)

- **Revisor:** `vanta-review` (sub-agente fresco) — ver §Notas si el spawn no está disponible; evidencia preparada para el orquestador.
- **Paths del diff:** `.github/workflows/ci-rust.yml`, `.lsan_suppressions`, `src/graph.rs`, `src/gds.rs`, `docs/dev/tasks/FIND-226.md` → **Tier Fast** (CI + tests + docs; sin paths adversariales — `src/storage/**` NO se toca).
- **Gate requerido:** `dev-tools/verify.ps1` ALL PASS (o evidencia mecánica equivalente: actionlint + fmt + clippy + nextest del scope) + veredicto registrado.
- **Checklist para el reviewer:**
  1. `actionlint .github/workflows/ci-rust.yml` → 0 issues.
  2. `git diff` → el job `sanitizer-asan` solo suma `llvm` + step de resolución; `sanitizer-tsan` intacto; 28 líneas de tests cambiadas a `&storage`.
  3. ¿Alguna firma pública cambió? No (verificar `git diff -- src/graph.rs src/gds.rs` solo en `mod tests`).
  4. Sanity: los 28 tests siguen ejerciendo el mismo comportamiento (los asserts no se tocaron).
  5. Contrato post-push con los comandos exactos del Step 7.
- **Veredicto: ✅ approve** — `vanta-review`, sesión `ses_f001fe8d0ffe1Mzzd30WRt18Zs` (contexto fresco ≠ autor; P2-01 satisfecho).
- **Verificación independiente del revisor:** `actionlint` exit 0 · `cargo fmt --check` exit 0 · nextest 55/55 **reproducido** · docs checks exit 0 · parseo independiente de ambos logs GH: todos los números duros coinciden (28×2728 B, 26×67108864 B, 999 bloques/751710 B, SUMMARYs, pre-existencia, ausencia de símbolos) · blob-hashes de los 5 paths estables (el diff revisado = estado actual) · `sanitizer-tsan` intacto.
- **Findings:** 0 Critical / 0 High / 0 Medium. Low #1 (error aritmético en la descomposición de allocations) → **corregido**; Low #2 (scope implícito del 55/55) → **aclarado**; Low #3 (error "metadata stub" sin output pegado) → nota honesta: transitorio, no reproducible tras completar el build; Low #4 (`documentation-skill` faltaba en SDP) → **cargada y aplicada** (frontmatter/kind verificados); Low #5 (varianza 26/28 especulativa) → wording ajustado a "causa no determinada".
- **Statements del revisor:** approach fix-vs-supresión = correcto (el `Box::leak` era objetivamente innecesario); evidencia suficiente y verificada; sin regresión ni violación del quality bar; sin outputs inventados. Riesgos post-push documentados (compat de versión del symbolizer; LSan honra `ASAN_SYMBOLIZER_PATH`; residual 999 bloques; higiene de commit — respetada: solo los 5 paths de FIND-226).

## DoD (3 niveles)

- **task:** contrato cumplido en sus tramos verificables hoy (clasificación + fix + job config) con tramos post-push documentados (§Step 7). El contrato NO pide "verde" — el job es best-effort por diseño.
- **commit:** conventional (`ci:`), atómico, verify mecánico local completo (commit LOCAL; sin push).
- **release:** n/a (best-effort; no gatea releases).

## Deuda técnica (Regla 6)

- **Introducida:** ninguna (0 dependencias nuevas; el package `llvm` es del runner, no del proyecto).
- **Pagada:** elimina la clase de falso-ruido dominante del job ASan (1.74 GB de leaks de test que ocultaban cualquier leak real) + hace operativas las supresiones LSAN (sin símbolos no matcheaban).

## Herramientas necesarias

- `gh` CLI (evidencia de runs), `actionlint` 1.7.12 (YAML), cargo/nextest (fix), OCR delegation (`dev-tools/ocr-review.ps1`).

**Skills cargadas (SDP):** `campaign-executor` + `progreso` (base, auto), `ci-cd-and-automation` (pinned CI), `git-workflow-and-versioning` (pinned CI), `deprecation-and-migration` (pinned storage/schema), `doubt-driven-development` (base type), `incremental-implementation` + `test-driven-development` (lifecycle BUILD), `documentation-skill` (obligatoria por crear `.md` bajo `docs/` — cargada tras review Low #4), `coordinated-web-search` (routing de investigación web — usado para validar symbolizer contra docs oficiales).

## Investigation Notes (web research — validado contra docs oficiales)

- **Symbolizer CLI:** clang docs (oficial) §"Symbolizing the Reports": *"set the `ASAN_SYMBOLIZER_PATH` environment variable to point to the `llvm-symbolizer` binary (or make sure `llvm-symbolizer` is in your `$PATH`)"* — https://clang.llvm.org/docs/AddressSanitizer.html (fetched 2026-10-03).
- **Ubuntu noble `llvm` provee `/usr/bin/llvm-symbolizer`** — filelist oficial: https://packages.ubuntu.com/noble/amd64/llvm/filelist (fetched 2026-10-03).
- **LSan suppressions:** formato `leak:<pattern>` matchea contra el **stack simbolizado**; `#` = comentario; `LSAN_OPTIONS=suppressions=` — https://clang.llvm.org/docs/LeakSanitizer.html (fetched 2026-10-03). → Refuerza que sin symbolizer las supresiones actuales no pueden matchear.
- **rustup `llvm-tools` NO trae llvm-symbolizer** (evidencia local: manifest del componente en `1.94.1-x86_64-pc-windows-msvc` lista solo `llvm-cov.exe`/`llvm-profdata.exe`) → se descarta esa vía.
- Runner ubuntu-latest (24.04) trae Clang 16/17/18 pre-instalados; se instala `llvm` explícito para no depender de esa ambigüedad (determinismo).

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — alcance real resuelto (evidencia de logs + código); residual a confirmar post-push es verificación, no incógnita |
| Pendientes de ejecución (downhill) | 1 — Step 7 post-push (orquestador/owner) |
| % completado | 87% (7/8 steps: 1-6 hechos + Step 6 cierre local; resta el post-push) |

## Notas

- **PROHIBIDO tocar (respetado):** `opencode.jsonc` (WIP ajeno), `docs/dev/plans/*` + `docs/pipeline-state.json` (orquestador), `benchmarks/*` + `perf-bench.yml` (FIND-232), job `sanitizer-tsan` (FIND-227 — secuenciado después en el mismo archivo: este commit va ANTES).
- **SECURITY:** no aplica — sin trust boundaries, sin inputs, sin dependencias del proyecto. Los tests no ejecutan red/FS externos nuevos.
- **PERFORMANCE:** no aplica a hot paths de producción; efecto medido: −1.74 GB de heap retenido en el proceso de tests (mejora, no regresión).
- **FIND-213 (NL_POOL):** candidato a verificar en el primer reporte simbolizado; si aparece, se registra/tria ahí.
- **Contexto percibido vs real:** el plan decía "~28 × 64 MB exactos = vfiles LSM no liberados al salir los tests" — confirmado con precisión: son vfiles **in-memory** (no la ruta LSM persistente `lsm.rs:156`, que es mmap) de engines **filtrados por tests** (no un drop faltante del engine).
