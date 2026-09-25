# B2a — Triaje `unwrap/expect` en producción (base de B2)

## 1. Descubrimiento (auto-detect tipo → codegraph blast radius → web si ambigüedad → baseline `/cleanCA <scope>`)
- Tipo: triaje/lectura (CERO cambios prod). Scope: `src/parser/mod.rs`, `src/physical_plan/mod.rs`, `src/wal.rs`, `src/wal_sharded.rs`, `src/engine.rs`, `src/sdk/api.rs` (+ resto de `src/`). Excluir `target/`, `benches`.
- Método: `rg -n 'unwrap\(\)|\.expect\(' src/` por archivo; frontera prod/test = línea de `mod tests` (+ `#[cfg(test)]` de archivo completo y `///` doc-comments). Archivos sin `mod tests` pero con matches solo en `///` → (a-doc). Binario `src/bin/crash_helper.rs` → (a-bin, herramienta de fuzz, no path servido).
- Categorías: (a) test · (b) invariante imposible documentable · (c) error real tragado → variante `Error` (`src/error.rs`) para B2b.
- Baseline: `cargo clippy -p vantadb -- --deny warnings` verde (verificado 2026-09-11, sin tocar prod).

## 2. Contrato (qué cambia / qué NO cambia / archivos exactos / comandos de verify)
- Cambia: SOLO este archivo `docs/dev/tasks/B2a.md` (tabla archivo:línea + categoría + variante Error para B2b).
- NO cambia: NINGÚN archivo bajo `src/` (prohibido código prod en B2a). No commitear (lo ejecuta el lead).
- Archivos exactos: lectura `src/**/*.rs` (6 focos obligatorios); escritura `docs/dev/tasks/B2a.md`.
- Verify: `cargo clippy -p vantadb -- --deny warnings` en bash directo (sin campaign MCP) + tabla cubre los 6 focos.

## 3. Steps atómicos
- [x] Slice 0: crear este task file (formato §4 del plan) — sin Read previo del inexistente (solo glob).
- [x] Slice 1: `rg` unwrap/expect en los 6 focos → todos (a). 607 ocurrencias, 0 prod.
- [x] Slice 2: `rg` resto de `src/` → 44×(c) + 15×(b), resto (a). Heurístico `mod tests` validado contra `^pub (fn|struct|…)` post-frontera (solo `vfile.rs` requirió corrección: frontera real `mod tests:463`, no `cfg:44`).
- [x] Slice 3: plan de fix por archivo para B2b + clippy verde + RESULTADO.

### Tabla A — 6 focos obligatorios (100% categoría (a), 0 prod)

| Archivo | Total | `mod tests` desde | Prod antes | Categoría |
|---|---|---|---|---|
| `src/parser/mod.rs` | 157 | L13 | 0 | (a) todo |
| `src/physical_plan/mod.rs` | 112 | L28 | 0 | (a) todo |
| `src/wal.rs` | 106 | L822 | 0 | (a) todo |
| `src/wal_sharded.rs` | 91 | L337 | 0 | (a) todo |
| `src/engine.rs` | 88 | L505 | 0 | (a) todo |
| `src/sdk/api.rs` | 53 | L26 | 0 | (a) todo |
| **Suma focos** | **607** | — | **0** | — |

### Tabla B — Categoría (c): error real tragado → fix en B2b (44 ocurrencias, 8 archivos)

| Archivo:Línea(s) | Código | Variante `Error` para B2b | Plan de fix B2b |
|---|---|---|---|
| `src/index/flat.rs:108,164,176,195,199` (5×) | `self.nodes.lock().unwrap()` (Mutex) | `Error::RuntimeError("flat index lock poisoned")` o nueva `Error::LockPoisoned` | `lock().map_err(...)` + `?`; test: envenenar mutex y assert Err. Hot path lectura → avisar a `vanta-tuner` |
| `src/index/scann.rs:184-187,225-228,289,301-302,308,320,323,328` (15×) | `*.lock().unwrap()` (min/max_bound, dim, entries, bounds_initialized) | `Error::RuntimeError` / `LockPoisoned` | Igual que flat.rs; agrupar helper `lock_or_poison()` si se repite >3× en el archivo |
| `src/index/diskann.rs:84-85,160-162,226,380,394,426,432-433,448` (12×) | `*.lock().unwrap()` (graph, vectors, medoid, bitsets) | `Error::RuntimeError` / `LockPoisoned` | Igual; hot path construcción/búsqueda → `vanta-tuner` |
| `src/index/diskann.rs:198-199` (2×) | `vectors.get(&a).unwrap()` / `get(&b).unwrap()` en sort de aristas reversas | `Error::NodeNotFound(id)` o `Error::Corruption("diskann neighbor missing vector")` | `ok_or_else` + `?`; test: vecino sin vector → Err, no panic |
| `src/sync_ext.rs:15,18,28` (3×) | `read()/write()/lock().expect("… poisoned")` | `Error::RuntimeError("… lock poisoned")` | Cambiar trait a devolver `Result`; actualizar callers (blast radius chico, archivo de 30 líneas) |
| `src/shred/mod.rs:215,232,241,259` (4×) | `data[o..o+N].try_into().unwrap()` (decode de fila) — el slicing ya puede panic antes | `Error::SerializationError` / `InvalidInput("shredded row truncated")` | Bounds-check explícito + `try_into().map_err` + `?`; test: fila truncada → Err |
| `src/planner.rs:356` (1×) | `join_spec.unwrap()` (plan JOIN sin spec) | `Error::InvalidInput("join missing spec")` o `SchemaError` | Validar en construcción del plan + test |
| `src/graph.rs:487` (1×) | `edges.get(&node.id).unwrap()` (adyacencia inconsistente) | `Error::Corruption` / `NodeNotFound(node.id)` | `ok_or_else` + `?`; test: nodo sin entrada de adyacencia → Err |
| `src/llm.rs:555` (1×, feature `remote-inference`) | `env::var("VANTA_OPENAI_API_KEY").expect(…)` en constructor prod | `Error::InvalidInput("VANTA_OPENAI_API_KEY must be set")` / `CliError` | `map_err` + `?` (constructor ya devuelve `Result`); test: env ausente → Err |

### Tabla C — Categoría (b): invariante imposible documentable → NO cambia semántica en B2b (15 ocurrencias, 9 archivos)

| Archivo:Línea(s) | Invariante (a documentar con `// SAFETY:`-style o `missing_panics_doc`) | Acción B2b |
|---|---|---|
| `src/crypto.rs:216,226,234,286,292` (5×) | AES-256-GCM infalible (garantía RustCrypto); clave de 32 bytes; `KDF_ITERATIONS != 0` — mensajes ya lo documentan | Solo añadir doc de invariante; sin cambio de firma |
| `src/index/serialize/bytes.rs:20` | `Write` sobre `Vec` infalible | Doc; sin cambio |
| `src/index/serialize/bytes.rs:406` | `chunks_exact(4)` ⇒ `try_into` infalible por construcción | Doc; sin cambio |
| `src/binary_header.rs:74` | `bytes.len() >= SIZE(16)` chequeado 6 líneas arriba ⇒ `[8..16]` mide 8 | Doc referencia al guard; sin cambio |
| `src/server/router.rs:272` | `rpm > 0 ⇒ burst_size, period_ms >= 1` ⇒ `GovernorConfigBuilder::finish()` infalible; expect es fail-closed intencional en startup | Doc + `debug_assert!`; sin cambio de firma (devuelve `Router`) |
| `src/server/state.rs:151` | `NonZero::new(1000)` const-no-cero infalible | Doc o `NonZero::new(1000).expect` → considerar `const`; sin cambio semántico |
| `src/storage/vfile_mmap.rs:532` | `Layout::from_size_align(len, 4)` en `Drop` (no puede devolver `Result`); `len` viene de alloc con align 4 | Doc; sin cambio (Drop no admite `?`) |
| `src/storage/vfile.rs:159` | OOM en constructor ya documentado como pánico intencional (equivale a abort de `Vec::with_capacity`) | Doc; sin cambio |
| `src/physical_plan/join.rs:95` | `current_left` es `Some` tras el `if is_none` + early-return del mismo loop | Doc de invariante; opcional `if let` defensivo |
| `src/cli_handlers/fmt.rs:22`, `src/cli_handlers/data.rs:69` | Template de spinner hardcodeado válido | Doc; sin cambio |

### Tabla D — Categoría (a): test / doctest / test-support (resto, conteo prod-real = 0)
- `mod tests` con matches: `executor.rs:30`, `backend.rs:43`, `fjall_backend.rs:36`, `rocksdb_backend.rs:30`, `in_memory.rs:13`, `archive.rs:54`, `vfile.rs:56` (L463+), `ops.rs:16`, `columnar.rs:17`, `audit.rs:17`, `error.rs:17`, `gc.rs:28`, `graph.rs:28` (L550+), `crypto.rs:19` (L433+), `shred:28` (L289+), `planner:43` (L456+), `binary_header:2`, `text_index.rs:2`, `hardware/mod.rs:2`, `cost_estimator.rs:5`, `gds.rs:12`, `governor.rs:7`, `query.rs:3`, `accumulator.rs:16`, `connection_pool.rs:9`, `config.rs:10` (L1348+), `ingestion.rs:4`, `migration.rs:3`, `integrations.rs:6`, `wal_shipping.rs:12`, `vector/governor.rs:2`, `node/*:12`, `wal:106`, `wal_sharded:91`, `engine:88`, `sdk/api:53`, `sdk/*:280+` aprox., `server/*`, `index/*`, `entity/*`, `physical_plan/*`, `storage/*`, `index/graph/*`, `index/search/*`.
- Archivos 100% test por path (sin `mod tests`, todo el archivo es test): `storage/engine/tests/*.rs` (728), `entity/*_tests.rs` (77), `skills/tests.rs` (48), `sdk/search/tests.rs` (46), `server/cli_server_auth_tests.rs` (21), `cli_server_auth_tests.rs` (19), `wiki/tests.rs` (39), `index/graph/tests.rs` (15), `index/search/tests.rs` (12), `testing/chaos.rs` (5).
- Archivo completo bajo `#[cfg(test)]`: `src/index/core.rs` (22).
- Doc-comments `///` (doctests, no runtime prod): `lib.rs:45-52` (5×), `config.rs:244-245` (2×), `sdk/builder.rs:66-67,95-96` (4×), `sdk/search/mod.rs:39-67` (5×), `sdk/api/namespaces.rs:332-335` (3×), `sdk/api/memory.rs:147-158,355-369,434-444` (15×).
- Binario de test-support (no path servido): `src/bin/crash_helper.rs:20,29` (2×) — herramienta de fuzz/crash-repro.
- `llm.rs:797-823` (5×) bajo `mod tests:780` (funciones `#[test]`, no compilan en prod aunque el `mod` no lleve `#[cfg(test)]` explícito).

### Conteo prod-real y plan B2b
- **(c) error real: 44 ocurrencias en 8 archivos** → orden B2b (empieza por WAL/engine según plan; aquí no hay WAL/engine prod — el riesgo está en índices): `sync_ext.rs` (3, blast radius mínimo, primero) → `planner.rs` + `graph.rs` + `join` (b, doc) → `shred` (4) → `llm.rs` (1, feature-gated) → `flat.rs`/`scann.rs`/`diskann.rs` (31 locks+gets, hot paths → coordinar con `vanta-tuner` + `vanta-chaos` stress tras el fix).
- **(b) invariante: 15 en 9 archivos** → solo documentación, mismo PR o previo a B2b.
- Regla 8 (eat-your-own): slices que toquen `Mutex`/`RwLock` en índices → delegar `vanta-chaos` + `vanta-review` antes de cerrar B2b.

## 4. Cierre (RESULTADO + `/cleanCA <scope>` PASS + recitation)
- RESULTADO abajo. `/cleanCA <scope>`: N/A código (triaje); gate mecánico clippy ✅.
- Recitation: objetivo B2a completo; estado done; última acción tabla escrita + clippy verde; contrato `cargo clippy -p vantadb -- --deny warnings` ✅ (Finished dev profile, 0 warnings); invariantes: `src/` intacto (`git status` solo muestra `docs/dev/tasks/B2a.md` nuevo); deuda: ninguna para B2a; próxima tarea B2b (fix 44×(c), orden §3).
