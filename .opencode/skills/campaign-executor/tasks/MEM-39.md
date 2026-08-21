# MEM-39 — Seed/import CLI (skills/persona iniciales)

Plan: `docs/plans/2026-08-21-vanta-context-engine.md` Task 4 · Ruta: vanta-worker
Contrato: `cargo check -p vanta-memory` pasa; CLI/subcomando importa un JSON de seed (skills + persona inicial) a namespaces sanitizados; idempotente por content-hash (patrón MEM-06/MEM-17); tests D19 con archivo temporal.
Stop condition: schema TDAM acoplado a Mongo/OpenClaw → schema propio mínimo documentado.

## Impacto mapeado (Regla 0)

**Archivos leídos completos (vía codegraph_explore verbatim):**
- TDAM ref `MemoryCore/src/core/seed/input.ts` (492L) — formato sessions/rounds/messages para importar CONVERSACIONES host-acopladas (OpenClaw capture); NO portable a skills+persona → **desviación documentada: schema propio mínimo** (stop condition del plan aplicada)
- `vanta-memory/src/core/skill/conversation_add/sink.rs` — `StoredSkill {name, description, content, content_hash, updated_at_ms}`, `SkillCoreSink` (ns `skills_extract/<scope>`), idempotencia por content-hash (DefaultHasher), `SkillSinkCounts`
- `vanta-memory/src/core/persona/persona_generator.rs` — `PersonaRecord {content, mode, generated_at_ms, generated_at}`, `PERSONA_KEY="persona.md"`, `persona_namespace()` (pub), `get_persona()` (pub), `write_persona` (privada → replicar put), `epoch_ms_to_rfc3339` en `core/prompts/l1_extraction.rs`
- `vanta-memory/src/core/conversation/l0_recorder.rs` — `sanitize_component(s,max,slash)` / `sanitize_key` (pub(crate))
- `vanta-memory/src/lib.rs` (48L), `vanta-memory/Cargo.toml` (33L) — deps serde/serde_json/thiserror/tracing/tempfile(dev); **sin serde_yaml en el workspace → JSON only (documentado)**
- `src/backend.rs:101` BackendKind (default Fjall), `src/storage/engine/init.rs:269-288` — error tipado claro si falta feature fjall
- `src/sdk/builder.rs` — `VantaEmbedded::open(path)` / `open_with_config`

**Referencias hacia dentro (entrantes):** módulo nuevo `seed/` — cero callers existentes. Blast radius = 1 línea en `lib.rs`.

**Referencias hacia afuera (salientes):** `vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemoryMetadata}`, `crate::core::conversation::{sanitize_component, sanitize_key}`, `crate::core::persona::persona_generator::{persona_namespace, get_persona, PERSONA_KEY}`, `crate::core::prompts::l1_extraction::epoch_ms_to_rfc3339`, `crate::core::conversation::now_ms`.

**Veredicto:** impacto BAJO. Módulo nuevo aislado + `pub mod seed;` en lib.rs + bin target nuevo (`src/bin/vanta-seed.rs`) + 1 feature passthrough en Cargo.toml.
**Desviación CLI:** `src/cli.rs` del crate core es IMPOSIBLE como glue — dependencia circular (vanta-memory → vantadb; Cargo prohíbe ciclo de paquetes). Glue = bin target propio de vanta-memory. Pre-mortem (2) del plan se cumple igual: parser en vanta-memory, bin es thin wrapper.

## Diseño

- **Schema propio mínimo** (JSON only — sin serde_yaml en workspace):
  ```json
  {
    "scope": "seed",              // opcional, default "seed" → ns skills_extract/<scope>
    "skills": [{"name","description","content"}],
    "persona": {"session_key", "content}   // opcional
  }
  ```
- **Persistencia:** skills → ns `skills_extract/<sanitize(scope)>`, key `sanitize_key(name)`, payload = StoredSkill JSON (paridad MEM-06 — los readers existentes lo leen). Persona → ns `persona/<sanitize(session_key)>`, key `persona.md`, payload = PersonaRecord JSON con mode=`first` (paridad get_persona).
- **Idempotencia:** skill existente con mismo content-hash → unchanged; persona con mismo content → unchanged. Counts `{created, updated, unchanged}` (patrón SkillSinkCounts).
- **Errores:** `SeedError` (thiserror): Io, Json, Validation(String), Vanta, Serde. Sin unwrap/expect.
- **CLI:** `cargo run -p vanta-memory --bin vanta-seed -- <seed.json> [--db <path>]`. Sin --db → InMemory + warning (no persiste). Con --db requiere feature `fjall` (passthrough `vanta-memory/fjall = ["vantadb/fjall"]`); sin feature → error tipado descriptivo del core.

## Steps

### Step 1 — Módulo seed + tests D19 ✅ DONE
- `src/seed/{mod,input}.rs` creados + `pub mod seed;` en lib.rs
- Schema propio mínimo JSON-only (desviación TDAM documentada en input.rs); StoredSkill/PersonaRecord payload parity con MEM-06/L3
- Tests `tests/seed.rs` 4/4 PASS + 2 unit tests del parser (import archivo temporal, replay idempotente, errores tipados, sanitización)
- Fix durante implementación: `SeedError::Persona(#[from] PersonaError)` (get_persona propaga PersonaError); raw string `r##"..."##` por `"#` en JSON de test

### Step 2 — CLI bin glue ✅ DONE
- `src/bin/vanta-seed.rs`: args `<seed.json> [--db <path>]`; sin --db → InMemory + warning; con --db → Fjall (requiere feature)
- Cargo.toml: feature passthrough `fjall = ["vantadb/fjall"]`
- Smoke test manual: import OK (`created=2`), archivo inexistente → error tipado + exit 1
- Verify: `cargo check -p vanta-memory --bins` exit 0

### Step 3 — Verify mecánico + cierre ✅ DONE
- `cargo check -p vanta-memory` → exit 0
- `cargo fmt --check -p vanta-memory` → exit 0
- `cargo clippy -p vanta-memory --all-targets` → 0 warnings propios (7 pre-existentes en vantadb core, fuera de blast radius)
- `cargo nextest run -p vanta-memory` → 395/395 PASS (389 previos + 6 nuevos)
- Sin commit (regla explícita de la invocación: NO commitear)

## Context Save Point
(ninguno — tarea completa)
