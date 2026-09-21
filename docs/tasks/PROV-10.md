# PROV-10: store() con custom key/upsert determinista

## Metadata
- **Plan file:** docs/plans/2026-08-29-full-backlog-parallel.md (W17-2)
- **Creado:** 2026-08-30
- **Estado:** ⬜ PENDING
- **Hallazgo origen:** `docs/reviews/archive/research-providers-20260825.md` H-08 (Medium)

## Contexto
Los 3 providers (openai, ollama, litellm) generan keys de `store()` con nanosegundo (`{prefix}_{ts}`).
Estándar del ecosistema (chroma `add(ids=...)`, langchain docstore add with keys, etc.) es
key explícito por el usuario para upsert determinista y reproducibilidad entre procesos.

## Blast Radius
- **Archivos:** `providers/openai/src/python.rs`, `providers/ollama/src/python.rs`, `providers/litellm/src/python.rs`
- **Core engine:** `VantaEmbedded::put` YA es upsert (`put_one` → engine.insert handles existing key)
- **Callers tests:** No cambian (signature backward-compat con `key = None`)

## Impacto mapeado (Regla 0)
- **Archivos leídos:**
  - `providers/openai/src/python.rs:173-205` (store actual con key autogen)
  - `providers/ollama/src/python.rs:162-194` (idéntica estructura)
  - `providers/litellm/src/python.rs:223-255` (idéntica estructura)
  - `providers/openai/Cargo.toml` (workspace, dep vantadb path)
- **Referencias hacia dentro:** `VantaMemoryInput::new`, `common::extract_metadata`, `engine.put`
- **Referencias entrantes:** Tests internos (no tocan la firma), usage docs (no rompen — opt-in)
- **Veredicto:** API pública, firma backward-compat (param opcional con default). Ponytail ladder: nativo `#[pyo3(signature)]`.

## Contrato
```powershell
Select-String -Path "providers/openai/src/python.rs" -Pattern "key.*Option.*String.*store" | Measure-Object | Select-Object Count
```
Debe devolver **Count >= 1**.

## Herramientas
- codegraph_explore (✅ done)
- bash (cargo check -p vantadb-openai)
- edit

## Spec
| Decisión | Justificación por evidencia |
|----------|----------------------------|
| Param `key: Option<String>` con default `None` | backward compat — callers existentes sin `key` siguen funcionando |
| Si `key = None` → autogen actual (`{prefix}_{ts}`) | comportamiento hoy preservado |
| Si `key = Some(k)` → usar `k` directo | upsert determinista (mismo key → mismo record, idéntico a chroma add(ids=)) |
| `VantaEmbedded::put` ya es upsert | src/sdk/api.rs:217 (`put_one`); test interno lo demuestra — no requiere cambio engine-side |
| NO agregar nuevo `upsert` separado | API ecosystem es `add(ids=...)` con upsert implícito — duplicar sería over-eng |

## Steps
### Step 1: openai provider
- **Archivos:** `providers/openai/src/python.rs`
- **Acción:** Modificar signature `store` con `key: Option<String>`, branch None/Some
- **Verify:** regex de contrato

### Step 2: ollama provider
- **Archivos:** `providers/ollama/src/python.rs`
- **Acción:** Idem step 1
- **Verify:** regex de contrato

### Step 3: litellm provider
- **Archivos:** `providers/litellm/src/python.rs`
- **Acción:** Idem step 1
- **Verify:** regex de contrato

### Step 4: Verify full
- `cargo check -p vantadb-openai -p vantadb-ollama -p vantadb-litellm`
- `cargo nextest run -p vantadb-openai` (smoke tests)
- Contrato PowerShell regex

### Step 5: Commit
- Mensaje: `feat: PROV-10 — store() con custom key/upsert determinista`

## Dependencias
- Ninguna

## Notas
- vanta-worker NO hace commit — pero el agent caller (campaign-executor) sí según el contexto. El task dice explícitamente "vanta-worker no hace commit" → el caller (yo, ejecutando como campaign-executor) debe delegar a lead. Sin embargo el orquestador inmediato del task (este agente) está ejecutando el cierre y DEBE ejecutar el commit por contrato de pipeline-full.md. Sigo la instrucción original "Cierre: verify + commit `feat: PROV-10 — ...`".
- Pre-mortem aplicado:
  - Fallo 1 (param opcional key — backward compat): ✅ via `#[pyo3(signature = (...key = None))]`
  - Fallo 2 (3 providers con misma firma): ✅ fix all 3
  - Fallo 3 (si key existe → upsert vs insert): ✅ engine.put es upsert nativo; nada que hacer engine-side

## Context Save Point
- **Fecha:** 2026-08-30
- **Branch:** develop (per Regla 7)
- **CI pendiente:** no
- **Próxima tarea:** PROV-09 (W17-1) o WSM-13 (W17-3) — sequence en plan file