# OLD-05: Search Quality v2 (Unicode folding + Snippets)

## Metadata
- **Plan file:** docs/plans/2026-07-24-backlog-triage-plan.md
- **Fuente:** docs/Backlog.md (línea 240)
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡
- **Tipo:** Mixto (Rust core + SDKs)
- **Turns estimados:** 15
- **Creado:** 2026-07-25T03:55:00Z
- **last-synced:** 2026-07-25T03:55:00Z
- **Estado:** ⏳ IN PROGRESS

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vantadb-ts`, `vantadb-wasm`, `src/sdk/search/mod.rs` |
| Callees | `src/sdk/search/snippet.rs`, `src/text_index.rs`, `src/tokenizer.rs` |
| Implicaciones | Mejora la precisión de snippets con caracteres acentuados/Unicode. No rompe contratos existentes de la API pública. |
| Riesgo | Bajo |

## Contrato
"cargo test --package vantadb --lib sdk::search::snippet pasa, cargo test --workspace pasa, y la generación de snippets con Unicode folding (acentos/diacríticos) resalta correctamente coincidencias como café/cafe"

## Herramientas necesarias
- cargo check / test
- rust-analyzer

## Solution Plan (Zero-code planning)
1. Mejorar `src/sdk/search/snippet.rs`:
   - Implementar helper de Unicode folding/diacritics removal ligero (e.g. mapping/folding de caracteres acentuados comunes a ASCII/base) para la localización y resaltado de coincidencia de snippets.
   - Actualizar `generate_snippet_with_highlighting` y `highlight_terms` para usar Unicode case/folding matching en lugar de únicamente `eq_ignore_ascii_case`.
2. Añadir tests unitarios exhaustivos para snippets con Unicode folding en `snippet.rs` (ej. "Café naïve résumé", "rápido", etc.).
3. Verificar la integración con WASM/TypeScript SDK y asegurar compilación limpia en todo el workspace.

## Steps

### Step 1: Implementar Unicode folding helper y resaltado robusto en `src/sdk/search/snippet.rs`
- **Archivos:** `src/sdk/search/snippet.rs`
- **Acción:** Agregar función de fold/normalización Unicode para comparación de términos y actualizar `highlight_terms` y `generate_snippet_with_highlighting`.
- **Verify:** `cargo test --package vantadb --lib sdk::search::snippet::tests`
- **Estado:** ⬜ PENDING

### Step 2: Añadir tests para Unicode Snippeting en `snippet.rs` y verificar workspace
- **Archivos:** `src/sdk/search/snippet.rs`
- **Acción:** Añadir tests unitarios para casos acentuados/multilingües.
- **Verify:** `cargo test --workspace`
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna.

## Notas
- `src/tokenizer.rs` ya cuenta con `AsciiFoldingFilter` vía Tantivy cuando la feature `advanced-tokenizer` está habilitada. Esta mejora en `snippet.rs` extiende la calidad de Unicode folding al generador de snippets sin requerir dependencias externas pesadas.
