# COMP-006: Edge Label Interning (String → u32 label_id)

## Metadata
- **Plan file:** Ninguno (tarea individual)
- **Fuente:** `docs/Backlog.md` línea 196
- **Esfuerzo:** 🟢 ~2d
- **Prioridad:** 🔴 Alta
- **Tipo:** Rust (core) + TypeScript (SDK types)
- **Turns estimados:** 15-45
- **Creado:** 2026-07-27
- **last-synced:** 2026-07-27
- **Estado:** ⬜ PENDING

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `src/storage/engine/ops.rs` (crea edges en insert/add_edge), `src/sdk/serialization/graph_types.rs` (VantaEdgeRecord::from), `src/sdk/types.rs` (EdgeRecord), tests en `tests/` |
| Callees | `src/node.rs` (Edge struct + UnifiedNode edges), `src/storage/engine/ops.rs` (edge insertion) |
| SDK surface | `vantadb-ts/src/types.ts:EdgeRecord.label` (string se mantiene — SDK no cambia), `vantadb-python/src/types.rs` (VantaEdgeRecord.label igual) |
| Serialización | Disk: Edge se serializa como parte de UnifiedNode → cambiar label field; se necesita migración de formato o backward compat |
| Tests | 8+ tests que crean edges con label: `test_edge_record_serialization_roundtrip`, `test_node_record_from_unified_node_*`, `test_recover_archived_nodes_*`, etc. |

### Implicaciones
- **API pública no cambia** — `VantaEdgeRecord.label: String` se mantiene. El interning es interno.
- **SDK TypeScript/TypeScript no cambia** — EdgeRecord.label sigue siendo string.
- **Serialización**: DiskNodeHeader solo tiene edge_count, los edges se escriben en bloque variable después del header fijo. Hay que cambiar la serialización de edges para usar `label_id: u32` y resolver el label via el interner en deserialización.
- **Riesgo medio**: cambios en serialización de disco requieren migración o versionado.
- **Performance**: impacto positivo (-80MB RAM para 1M edges). Resolve es O(1) hashmap lookup.

## Contrato
```
cargo nextest run --profile audit --workspace --build-jobs 2 pasa
  && `Edge.label` deja de existir (solo `label_id: u32`)
  && `VantaEdgeRecord.label` sigue siendo `String` (compatibilidad SDK)
  && `cargo check --workspace` pasa sin warnings
  && `cargo clippy --workspace --all-targets -- -D warnings` pasa
  && pruebas de serialización roundtrip pasan
```

## Herramientas necesarias
- cargo-mcp (check, clippy, fmt, test)
- rust-analyzer-mcp (diagnostics)
- codegraph_explore (blast radius)

## Investigation Notes
- **Enfoque elegido (ponytail):** HashMap<String, u32> + Vec<String> manual. La cardinalidad de edge labels suele ser baja (decenas o cientos, no millones). No vale la pena agregar una dependencia externa como `lasso` o `string-interner`.
- **LabelIntern** struct con `intern(label: &str) -> u32` y `resolve(id: u32) -> &str`, almacenado en `StorageEngine`.
- **Serialización**: LabelIntern persistido como un map en una columna separada (InternalMetadata) para que sobreviva reinicios.
- **Alternativa descartada**: usar `lasso::Rodeo` — buena crate pero añade dependencia para algo que un HashMap simple resuelve.

## Steps

### Step 1: Crear LabelIntern struct
- **Archivos:** `src/node.rs` (agregar struct + impl)
- **Acción:** Agregar `LabelIntern` con:
  ```rust
  pub(crate) struct LabelIntern {
      map: HashMap<String, u32>,
      strings: Vec<String>,
  }
  ```
  Métodos: `new()`, `intern(&mut self, label: &str) -> u32`, `resolve(&self, id: u32) -> Option<&str>`, `lookup(&self, label: &str) -> Option<u32>`, `len() -> usize`
- **Verify:** `cargo check -p vantadb`

### Step 2: Cambiar Edge.label → label_id
- **Archivos:** `src/node.rs`
- **Acción:**
  - `Edge.label: String` → `Edge.label_id: u32`
  - `Edge::new(target, label_id)` → toma u32
  - `Edge::with_weight(target, label_id, weight)` → toma u32
  - Actualizar `Edge::new(..)`, `Edge::with_weight(..)` signatures
- **Verify:** `cargo check -p vantadb`

### Step 3: Agregar LabelIntern a StorageEngine
- **Archivos:** `src/storage/mod.rs` o `src/engine.rs`
- **Acción:**
  - Agregar `pub(crate) label_intern: LabelIntern` como campo en `StorageEngine`
  - Inicializar en `StorageEngine::open()` y `new()`
  - Agregar método `StorageEngine::intern_label(&mut self, label: &str) -> u32`
  - Agregar método `StorageEngine::resolve_label(&self, id: u32) -> Option<&str>`
- **Verify:** `cargo check -p vantadb`

### Step 4: Actualizar insert/add_edge en ops.rs
- **Archivos:** `src/storage/engine/ops.rs`
- **Acción:**
  - Buscar todas las creaciones de `Edge { target, label, weight }`
  - Cambiar `label: "string".into()` por `label_id: engine.intern_label("string")`
  - `add_edge` debe internar el label
  - `insert` debe internar labels de edges
- **Verify:** `cargo check -p vantadb`

### Step 5: Actualizar VantaEdgeRecord (SDK mantiene label String)
- **Archivos:** `src/sdk/serialization/graph_types.rs`
- **Acción:**
  - El `From<UnifiedNode> for VantaNodeRecord` mapea `edge.label` a `VantaEdgeRecord.label`
  - Cambiar: en vez de `label: edge.label`, resolver desde el interner:
    ```rust
    label: engine.resolve_label(edge.label_id).unwrap_or("").to_string()
    ```
  - Esto requiere pasar una referencia a `StorageEngine` al `From` impl, o cambiar el approach
  - **Opción ponytail:** pasar el `LabelIntern` como ref, o agregar un método `UnifiedNode::resolve_edge_label()` que tome el interner
- **Verify:** `cargo check -p vantadb`

### Step 6: Persistir LabelIntern
- **Archivos:** `src/storage/engine/mod.rs` o archivo nuevo
- **Acción:**
  - Al guardar: serializar `LabelIntern` como JSON o binario en `InternalMetadata`
  - Al cargar: restaurar desde `InternalMetadata`
  - Esto asegura que los `label_id` sobrevivan reinicios
- **Verify:** `cargo check -p vantadb`

### Step 7: Actualizar tests
- **Archivos:** `src/sdk/serialization/graph_types.rs` (tests), `src/node.rs` (tests), `tests/`
- **Acción:**
  - Tests que crean `Edge { label: "..." }` → cambiar a `label_id: engine.intern_label("...")`
  - Test de serialización roundtrip de VantaEdgeRecord mantiene `label: String`
  - Verificar que tests de UnifiedNode edges compilan
- **Verify:** `cargo nextest run --profile audit --workspace --build-jobs 2`

### Step 8: TypeScript SDK — no cambia
- **Archivos:** `vantadb-ts/src/types.ts`
- **Acción:** Verificar que `EdgeRecord.label: string` se mantiene (no tocar)
- **Verify:** `cd vantadb-ts && npx tsc --noEmit`

### Step 9: fmt + clippy final
- **Acción:** `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings`
- **Verify:** Pasa sin errores

## Dependencias
- Ninguna

## Notas
- El contrato de serialización cambia: edges en disco ahora llevan `label_id: u32` en vez de `label: String`. Esto rompe compatibilidad con archivos existentes. Considerar agregar un version de formato o migración.
- Ponytail: HashMap simple alcanza. Si la cardinalidad de labels supera 100K únicos, considerar `lasso` o `ustr`.
- SDK público no se ve afectado — `VantaEdgeRecord.label` sigue siendo `String`.