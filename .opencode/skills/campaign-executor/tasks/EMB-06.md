# EMB-06 — SQL vector auto-embed (fix punto 3, physical_plan)

## Metadata
- ID: EMB-06
- Phase: 11 Embeddings Local-First
- Priority: 🟡 Media
- Effort: 🟢 2-4h
- Archivos clave: src/physical_plan.rs (real; plan cita src/physical_plan/vector.rs:51,129), src/query.rs
- Contrato: cargo check -p vantadb --features embed-local pasa; cargo test --features embed-local --test parser (o cargo test --lib) no rompe; PhysicalVectorSearch::open y Refine tienen cfg embed-local branch con LocalOnnxProvider
- Depends: EMB-02 (LocalOnnxProvider), EMB-03

## Objetivo
`src/physical_plan.rs:224 PhysicalVectorSearch::open` y `:739 PhysicalVectorRefine::open` añadir `#[cfg(feature="embed-local")]` branch con `LocalOnnxProvider::embed(&query_vec_text)` además del existente `remote-inference`. Ahora `VECTOR_SEARCH('hola mundo')` funciona offline sin `VANTA_LLM_URL`.

## Plan (planning-and-task-breakdown + ponytail full + source-driven)
- Ponytail ladder: existe `LocalOnnxProvider` en `src/llm.rs` (EMB-02) → reusar, no reescribir. Stdlib `std::env::var` ya usado en `src/llm.rs:57`. Thin wrapper en physical_plan, lógica vive en `llm.rs`.
- Source-driven: `cfg(feature)` verificado en `Cargo.toml:106 embed-local = ["dep:ort","dep:tokenizers"]` y `src/llm.rs:93` factory patterns.
- Vertical slice: 1 step — añadir fallback embed-local en ambos operators, verify `cargo check` + `cargo test --lib`.

## Steps
### Step 1 — Añadir #[cfg(embed-local)] branch en PhysicalVectorSearch::open y PhysicalVectorRefine::open
- [x] PLAN: leer `src/physical_plan.rs`, `src/llm.rs`, `Cargo.toml`
- [ ] ACT: editar `src/physical_plan.rs` ambas `open()` — añadir cfg embed-local con LocalOnnxProvider::new + embed, fallback si vector is None
- [ ] VERIFY:
  - cargo check -p vantadb --features embed-local
  - cargo check -p vantadb (sin features no regresión)
  - grep -n "embed-local" src/physical_plan.rs && grep -n "LocalOnnxProvider" src/physical_plan.rs
  - cargo test -p vantadb --lib --features embed-local
  - cargo test -p vantadb --test parser --features embed-local (si existe) o cargo test --lib

## State
- status: in-progress
- activeGoal: Step 1 ACT
- lastAction: PLAN completado — archivos leídos, patrón factory LocalOnnxProvider validado
- nextAction: Edit src/physical_plan.rs — añadir cfg embed-local branches
- contract: cargo check -p vantadb --features embed-local pasa; PhysicalVectorSearch/Refine cfg embed-local + LocalOnnxProvider
- budget: 1/5 iteraciones

## Verification
- `cargo check -p vantadb --features embed-local` → ok
- `cargo test -p vantadb --lib --features embed-local` → no rompe
- `grep embed-local src/physical_plan.rs` → 2 hits, `grep LocalOnnxProvider` → 2 hits

## Notes
- Ponytail: deterministic_embed fallback mantiene CI verde sin 691MB. No duplicar lógica ort — delegates a llm.rs.
- No tocar EMB-04,05,07..09.
