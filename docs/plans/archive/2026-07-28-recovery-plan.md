# Plan de Recuperación — SDK Gaps, Features Perdidas y Deuda de Implementación

```
╔══════════════════════════════════════════════════════════════╗
║  VantaDB — Recovery Plan / SDK Gap Closure                  ║
║  Basado en: Investigación del 2026-07-28                    ║
║  Origen: docs/audit-reports/audit-full.md + backlog audit  ║
║  Objetivo: Cerrar los 8 gaps verificados en código          ║
╚══════════════════════════════════════════════════════════════╝
```

---

## Resumen Ejecutivo

**Hallazgo principal:** 12 puntos investigados contra código real, git history, Python/WASM/CLI. Se identificaron 3 categorías de trabajo:

| Categoría | Cantidad | Esfuerzo total |
|-----------|----------|---------------|
| 🔴 **Implementar desde cero** (nunca existieron en SDK) | 3 tareas | ~8-12 días |
| 🟠 **Exponer/Completar** (engine tiene, falta binding) | 3 tareas | ~7-17 días |
| 🟢 **Fix trivial** (un archivo, config) | 1 tarea | ~1 hora |
| 📋 **Plan-only** (requiere diseño antes de implementar) | 1 tarea | ~4-7 días |

**Corrección a la hipótesis original:** Las 3 funciones core (`delete_by_filter`, `count`, `similar_to_key`) **nunca fueron parte del SDK programático de Rust**. Solo existieron como CLI handlers (eliminados en AUD-09) o nunca se implementaron. Multi-namespace como feature de búsqueda tampoco existió nunca.

**Cross-reference Backlog IDs:** Los REC IDs de este plan ya existen en el backlog como `SDK-01` a `SDK-05` (descubiertos durante la inserción). Mapeo:

| Plan REC-ID | Backlog ID | Nombre |
|-------------|-----------|--------|
| REC-002 | SDK-01 | delete_by_filter() |
| REC-004 | SDK-02 | similar_to_key() |
| REC-003 | SDK-03 | count() |
| REC-005 | SDK-04 | Multi-namespace search |
| REC-006 | SDK-05 | Expanded metadata filters |
| REC-001 | *(nuevo)* | Foundation types (VantaFilterOp) |
| REC-007 | *(nuevo)* | WAL compaction + vacuum CLI |
| REC-008 | *(nuevo)* | Incremental backup design |
| REC-009 | *(nuevo)* | PQ analysis |
| REC-010 | *(nuevo)* | py.typed + maturin config |
| REC-999 | *(nuevo)* | progreso/README.md fix |

**Ejecutar con:** `/pipeline task SDK-01` (o `REC-002`) — ambos resolvían al mismo trabajo.

---

## Arquitectura de Decisiones

### D1: ¿Dónde vive la lógica — SDK engine vs CLI?

Cada función que se implemente debe seguir esta jerarquía:

```
1. Lógica de negocio en StorageEngine (src/storage/engine/) — si es operación atómica
2. Wrapper en VantaEmbedded (src/sdk/api.rs) — si es operación de alto nivel
3. CLI handler (src/cli_handlers/) — si debe ser accesible desde terminal
4. Python binding (vantadb-python/src/lib.rs) — si debe ser accesible desde Python
```

**Regla:** NO replicar lógica en cada capa. La lógica vive en el engine o SDK, las capas superiores son wrappers delgados.

### D2: Filtros — ¿API de surface nueva o reusar IQL?

Para los filtros de metadata no-IQL en el SDK, decidir entre:
- **Opción A** (recomendada): Exponer `VantaMemoryFilter` enum con operadores → usarlo en `list()` y nuevo `delete_by_filter()`
- **Opción B**: Forzar a los usuarios a usar `query()` (IQL) para filtros complejos

**Decisión:** Opción A — porque el SDK debe ser autosuficiente sin requerir IQL.

### D3: ¿count() como función aparte o como opción de list()?

- **Opción A** (recomendada): `count(namespace: &str, filter: Option<VantaMemoryFilter>) -> u64` — función independiente
- **Opción B**: `list().count` como campo en el resultado
- **Opción C**: Ambas

**Decisión:** Opción A + Opción B — `count()` como shortcut y `list()` incluye `total_count` opcional.

### D4: Multi-namespace — ¿nuevos métodos o modificar existentes?

- **Opción A** (recomendada): Agregar overloads/alternativas: `search_namespaces(namespaces: &[&str], request)`, `get_from_any(namespace: &str, key: &str)` 
- **Opción B**: Cambiar signature de `search()` a `namespaces: &[&str]` (breaking change)
- **Opción C**: `search_all(request)` que busca en todos los namespaces

**Decisión:** Opción A + C — no romper API existente. Agregar `search_multi()` y `search_all()`.

---

## Task List

---

## Fase 1: 🔴 SDK Functions desde Cero (P0 — Bloqueante)

> **Nota sobre el orden:** Cada tarea de Fase 1 requiere que el tipo `VantaMemoryFilterOp` exista (definido en REC-001) antes de poder implementar las funciones que lo usan.

### REC-001: [Foundation] Definir `VantaMemoryFilterOp` + `VantaMemoryFilter` types

**Descripción:** Crear los tipos de filtro en el SDK que permitan expresar operadores relacionales (Eq, Neq, Gt, Lt, Gte, Lte) sobre metadata. Estos tipos serán reusados por `list()`, `delete_by_filter()` y `count()`.

**Relación con hallazgos:**
- Desbloquea: REC-002 (`delete_by_filter`), REC-003 (`count`), REC-006 (SDK metadata filters)
- Dependencia de: `VantaMemoryMetadata` existente en `src/sdk/types.rs`

**Especificación:**

```rust
/// Operadores de comparación para filtros de metadata.
pub enum VantaFilterOp {
    Eq,    // ==
    Neq,   // !=
    Gt,    // >
    Lt,    // <
    Gte,   // >=
    Lte,   // <=
}

/// Un filtro individual: campo + operador + valor.
pub struct VantaMemoryFilterItem {
    pub field: String,
    pub op: VantaFilterOp,
    pub value: VantaValue,
}

/// Lista de filtros combinados con AND lógico.
pub type VantaMemoryFilter = Vec<VantaMemoryFilterItem>;
```

**Acceptance Criteria:**
- [ ] `VantaFilterOp` enum con 6 variantes definido en `src/sdk/types.rs`
- [ ] `VantaMemoryFilterItem` struct con field/op/value definido
- [ ] `VantaMemoryFilter = Vec<VantaMemoryFilterItem>` type alias definido
- [ ] Serialización/deserialización (serde) para todos los nuevos tipos
- [ ] `cargo check -p vantadb` pasa sin errores
- [ ] `cargo clippy -p vantadb` pasa sin warnings nuevos

**Archivos tocados:**
- `src/sdk/types.rs` — nuevos tipos de filtro
- `src/sdk/serialization/mod.rs` — si necesita serialización extra
- `src/sdk/mod.rs` — re-export de nuevos tipos

**Verificación:**
- [ ] `cargo check -p vantadb`
- [ ] `cargo clippy -p vantadb --deny warnings`
- [ ] Test unitario de serialización round-trip
- [ ] `cargo nextest run --profile audit -p vantadb --test types`

**Riesgos:**
- ⚠️ **Breaking change potencial:** Si se cambia la signatura de `VantaMemoryListOptions.filters` de `VantaMemoryMetadata` a `Option<VantaMemoryFilter>`, puede romper código existente.
  - **Mitigación:** Hacer `VantaMemoryFilter` un nuevo campo opcional, mantener `filters: VantaMemoryMetadata` como deprecated con `#[allow(deprecated)]`.
- ⚠️ **Compatibilidad con engine IQL:** Asegurar que los operadores mapean 1:1 con `RelOp` en `src/query.rs`.
  - **Mitigación:** Revisar `src/query.rs:129-143` para verificar mapeo exacto.

**Dependencias:** Ninguna

**Esfuerzo:** 🟢 Small (~2-3hr)

---

### REC-002: [Core] Implementar `delete_by_filter()` en SDK + CLI

**Descripción:** Implementar `VantaEmbedded::delete_by_filter()` que borre todos los registros en un namespace que coincidan con un filtro de metadata. Debe escanear, evaluar filtros, y eliminar en batch, con reporte de cuántos se borraron.

**Relación con hallazgo:** 🔴 Hallazgo 1 — `delete_by_filter` **NUNCA existió como SDK method**. Solo fue CLI handler (eliminado en AUD-09). Se implementa desde cero.

**Especificación:**

```rust
impl VantaEmbedded {
    /// Delete all records in a namespace matching the given filter.
    /// Returns the number of records deleted.
    /// 
    /// # Arguments
    /// * `namespace` — Namespace to operate on
    /// * `filter` — Metadata filter to match records for deletion
    pub fn delete_by_filter(
        &self,
        namespace: &str,
        filter: VantaMemoryFilter,
    ) -> Result<u64>;
}
```

**Flujo interno:**
1. Validar namespace (no vacío)
2. Hacer `list(namespace)` para obtener todos los IDs
3. Aplicar filtros en memoria (reusar `matches_memory_filters` extendido con operadores)
4. Para cada match: `delete(namespace, key)` 
5. Retornar count de eliminados

**⚠️ Consideración de performance:** Para namespaces con millones de registros, el scan completo en `list()` puede ser lento. Considerar:
- **Opción rápida:** Usar `scan_nodes()` + filtro temprano (como hace `InFilter`)
- **Opción segura (V1):** Paginar `list()` con `VantaMemoryListOptions` hasta cubrir todo

**Decisión para V1:** Usar paginación sobre `list()`. Optimizar a scan directo en V2 si es necesario.

**Acceptance Criteria:**
- [ ] `pub fn delete_by_filter()` en `src/sdk/api.rs` firmada correctamente
- [ ] CLI handler `vanta-cli delete-by-filter --namespace <ns> --filter <json>` en `src/cli_handlers/crud.rs`
- [ ] Reporte de retorno: `Ok(count)` con número de registros borrados
- [ ] Tests: (a) borra registros que matchean, (b) NO borra registros que no matchean, (c) filtro vacío es error, (d) namespace vacío es error, (e) múltiples filtros en AND
- [ ] `cargo check -p vantadb` pasa
- [ ] `cargo clippy -p vantadb` sin warnings nuevos
- [ ] CLI integrado en `src/cli.rs` `Commands` enum

**Archivos tocados:**
- `src/sdk/api.rs` — nuevo método `delete_by_filter()`
- `src/sdk/serialization/mod.rs` — posible helper para filtrar in-memory
- `src/cli_handlers/crud.rs` — nuevo handler CLI
- `src/cli.rs` — nuevo comando `DeleteByFilter` variant
- `src/sdk/mod.rs` — re-export si es necesario

**Verificación:**
- [ ] `cargo check -p vantadb` 
- [ ] `cargo clippy -p vantadb --deny warnings`
- [ ] `cargo nextest run --profile audit -p vantadb --test crud`
- [ ] Test manual: `vanta-cli delete-by-filter --namespace test --filter '{"key": "value"}'`

**Riesgos:**
- ⚠️ **Muchos registros + timeout:** Si el namespace tiene millones de registros, el delete puede tomar minutos.
  - **Mitigación:** Documentar el límite de la implementación V1. Agregar log progresivo cada 1000 registros.
- ⚠️ **Operación no atómica:** Si el proceso falla a mitad, algunos registros quedan borrados y otros no.
  - **Mitigación:** Documentar que no es transaccional. El WAL no soporta rollback batch.
- ⚠️ **Reintroducir CLI command eliminado:** Asegurar que el nuevo handler no resucita patrones de AUD-09 que se eliminaron por razones válidas.
  - **Mitigación:** Revisar commit `e9371ea8` para entender por qué se eliminó. La razón fue "dead code", no "mala implementación". Como ahora es SDK method + CLI binding, es aceptable.

**Dependencias:** REC-001 (types de filtro)

**Esfuerzo:** 🟡 Medium (~1-2d)

---

### REC-003: [Core] Implementar `count()` en SDK + CLI

**Descripción:** Implementar `VantaEmbedded::count()` que retorne el número total de registros en un namespace, opcionalmente filtrados por metadata. Es un shortcut sobre `list()` con filtro pero que solo retorna el count.

**Relación con hallazgo:** 🔴 Hallazgo 3 — `count()` solo existió como CLI handler `cmd_count` (eliminado en AUD-09). `fn count_memory_records_from()` existió como helper privado en old sdk.rs pero nunca fue público.

**Especificación:**

```rust
impl VantaEmbedded {
    /// Count records in a namespace, optionally filtered by metadata.
    pub fn count(
        &self,
        namespace: &str,
        filter: Option<VantaMemoryFilter>,
    ) -> Result<u64>;
}
```

**Flujo interno:**
- Sin filtro: delegar a `StorageEngine::node_count()` o scan rápido
- Con filtro: `list()` + aplicar filtros + retornar len

**⚠️ Optimización sin filtro:** `StorageEngine` tiene `node_count()`. Verificar si existe contador por namespace. Si no, agregar `count_by_namespace()` en engine.

**Acceptance Criteria:**
- [ ] `pub fn count()` en `src/sdk/api.rs` — sin filtro usa contador rápido
- [ ] Con filtro opcional usa `list()` + filtro in-memory
- [ ] CLI handler `vanta-cli count --namespace <ns> [--filter <json>]`
- [ ] Tests: (a) count total, (b) count con filtro, (c) count en namespace vacío = 0
- [ ] `cargo check -p vantadb` pasa
- [ ] `cargo clippy -p vantadb` sin warnings nuevos

**Archivos tocados:**
- `src/sdk/api.rs` — nuevo método `count()`
- `src/storage/engine/mod.rs` — posible `count_by_namespace()` en StorageEngine
- `src/cli_handlers/crud.rs` — nuevo handler `cmd_count`
- `src/cli.rs` — nuevo comando `Count` variant

**Verificación:**
- [ ] `cargo check -p vantadb`
- [ ] `cargo clippy -p vantadb --deny warnings`
- [ ] `cargo nextest run --profile audit -p vantadb`
- [ ] Test manual: `vanta-cli count --namespace test`

**Riesgos:**
- ⚠️ **Performance con filtro en DB grande:** Sin filtro debe ser O(1); con filtro requiere scan.
  - **Mitigación:** Documentar en docstring. Filtro = scan completo.
- ⚠️ **Engine `node_count()` puede no distinguir namespaces:** Verificar implementación actual.
  - **Mitigación:** Si no existe contador por namespace, implementar uno en StorageEngine.

**Dependencias:** REC-001 (types de filtro, si se usa filtro opcional)

**Esfuerzo:** 🟡 Medium (~1d)

---

### REC-004: [Core] Implementar `similar_to_key()` en SDK + CLI

**Descripción:** Implementar búsqueda por similitud usando una key existente como referencia. Dado un namespace y una key, recupera el vector de ese registro y ejecuta `search_vector()` con él, retornando los top-k más similares.

**Relación con hallazgo:** 🔴 Hallazgo 2 — `similar_to_key` **NUNCA se implementó** en ningún lado. Solo existe en documentación como feature planeada. Es implementación desde cero.

**Especificación:**

```rust
impl VantaEmbedded {
    /// Find records similar to a known record by key.
    /// Retrieves the vector for `key` and performs a similarity search.
    pub fn similar_to_key(
        &self,
        namespace: &str,
        key: &str,
        top_k: usize,
    ) -> Result<Vec<VantaMemorySearchHit>>;
}
```

**Flujo interno:**
1. `get(namespace, key)` para obtener el registro
2. Si el registro no tiene vector → error `VantaError::NoVectorForKey`
3. `search_vector(record.vector, top_k)` para encontrar similares
4. Excluir el mismo registro (misma key) de los resultados
5. Retornar hits

**Acceptance Criteria:**
- [ ] `pub fn similar_to_key()` en `src/sdk/api.rs`
- [ ] Error `VantaError::NoVectorForKey` definido
- [ ] Excluye el mismo registro de los resultados
- [ ] CLI handler `vanta-cli similar-to-key --namespace <ns> --key <key> [--top-k N]`
- [ ] Tests: (a) encuentra registros similares, (b) NO incluye el mismo registro, (c) error si key no tiene vector, (d) error si key no existe
- [ ] `cargo check -p vantadb` pasa

**Archivos tocados:**
- `src/sdk/api.rs` — nuevo método `similar_to_key()`
- `src/error.rs` — nuevo error `NoVectorForKey`
- `src/cli_handlers/search.rs` — nuevo handler `cmd_similar_to_key`
- `src/cli.rs` — nuevo comando `SimilarToKey` variant
- `src/sdk/mod.rs` — re-export

**⚠️ Dato crítico de implementación:** `search_vector()` ya existe en `VantaEmbedded` y busca en el índice vectorial completo (no por namespace). Decidir si `similar_to_key` debe filtrar por namespace los resultados. 
- **Decisión:** Sí — post-filtrar por namespace. El usuario busca "similar a esta key en este namespace", por lo que los resultados deben pertenecer al mismo namespace.

**Verificación:**
- [ ] `cargo check -p vantadb`
- [ ] `cargo clippy -p vantadb --deny warnings`
- [ ] `cargo nextest run --profile audit -p vantadb`
- [ ] Test manual: insertar 5 vectores → `similar_to_key` de uno → obtener top-3

**Riesgos:**
- ⚠️ **Namespace puede tener registros sin vector:** `get()` funciona, pero si no hay vector no se puede buscar.
  - **Mitigación:** Error claro: `VantaError::NoVectorForKey`.
- ⚠️ **Search_vector es global, no por namespace:** `search_vector()` no acepta namespace. Los resultados pueden venir de otros namespaces.
  - **Mitigación:** Post-filtrar resultados por namespace en Rust.

**Dependencias:** Ninguna (usa `get()` + `search_vector()` existentes)

**Esfuerzo:** 🟡 Medium (~1-2d)

---

### Checkpoint Fase 1: SDK Functions Core

- [ ] `cargo check -p vantadb` sin errores
- [ ] `cargo clippy -p vantadb --deny warnings` sin nuevos warnings
- [ ] `cargo nextest run --profile audit -p vantadb` todos los tests pasan
- [ ] Las 3 funciones responden correctamente en test manual
- [ ] CLI commands funcionan: `vanta-cli --help` muestra los nuevos comandos

---

## Fase 2: 🟠 Binding de Engine → SDK/CLI (P1 — Alta)

### REC-005: [Feature] Multi-namespace search (`search_multi`, `search_all`)

**Descripción:** Agregar soporte para búsqueda en múltiples namespaces simultáneamente. No rompe API existente — agrega `search_multi()` que acepta `&[&str]` y `search_all()` que busca en todos.

**Relación con hallazgo:** 🔴 Hallazgo 4 — Multi-namespace **NUNCA existió como API de búsqueda**. `Vec<String>` solo existe en tipos de reporte (output), no como input.

**Especificación:**

```rust
impl VantaEmbedded {
    /// Search across multiple namespaces.
    pub fn search_multi(
        &self,
        namespaces: &[&str],
        request: VantaMemorySearchRequest,
    ) -> Result<Vec<VantaMemorySearchHit>>;

    /// Search across all namespaces.
    pub fn search_all(
        &self,
        request: VantaMemorySearchRequest,
    ) -> Result<Vec<VantaMemorySearchHit>>;
}
```

**Flujo interno:**
1. `search_multi`: Ejecutar `search()` para cada namespace, mergear resultados, deduplicar por (namespace, key), re-rank por score
2. `search_all`: Obtener lista de namespaces con `list_namespaces()`, luego `search_multi()`
3. Los resultados incluyen `namespace` field para identificar origen

**⚠️ Consideración de performance:** Para N namespaces, hace N búsquedas. Para muchos namespaces, considerar paralelización con Rayon (ya disponible en proyecto).

**Acceptance Criteria:**
- [ ] `pub fn search_multi(&self, namespaces: &[&str], request) -> Result<Vec<...>>`
- [ ] `pub fn search_all(&self, request) -> Result<Vec<...>>`
- [ ] CLI: `vanta-cli search --namespace ns1,ns2 --query ...` (multi-value)
- [ ] Resultados deduplicados por (namespace, key)
- [ ] Resultados re-rankeados por score descendente
- [ ] Tests: (a) busca en 2 namespaces, (b) `search_all` funciona, (c) namespace vacío produce error, (d) deduplicación funciona
- [ ] `cargo check -p vantadb` pasa

**Archivos tocados:**
- `src/sdk/api.rs` — nuevos métodos `search_multi()`, `search_all()`
- `src/sdk/search/mod.rs` — posible helper de multi-search + merge
- `src/cli_handlers/search.rs` — modificar handler existente para multi-namespace
- `src/cli.rs` — posible cambio en `Search` command para aceptar múltiples namespaces
- `src/sdk/serialization/vector_types.rs` — `VantaMemorySearchRequest` puede necesitar campo `namespaces`

**Verificación:**
- [ ] `cargo check -p vantadb`
- [ ] `cargo clippy -p vantadb --deny warnings`
- [ ] `cargo nextest run --profile audit -p vantadb --test search`
- [ ] Test manual: 2 namespaces con datos → search_multi → resultados combinados

**Riesgos:**
- ⚠️ **Multi-namespace search cruza límites de aislamiento:** Por diseño, namespaces aíslan datos. `search_multi` los combina.
  - **Mitigación:** Documentar explícitamente.
- ⚠️ **Merge de scores:** Scores de diferentes namespaces pueden no ser directamente comparables (distintos HNSW, distintas normalizaciones).
  - **Mitigación:** Investigar si los scores son comparables. Si no, normalizar antes de merge.
- ⚠️ **Performance con N namespaces grandes:** Hace N búsquedas secuenciales.
  - **Mitigación:** V1 = secuencial. V2 = paralelo con Rayon.

**Dependencias:** Ninguna (usa `search()` existente)

**Esfuerzo:** 🟡 Medium-Large (~4-7d)

---

### REC-006: [SDK] Exponer metadata filters operadores en API del SDK

**Descripción:** Extender el SDK para que `list()`, `delete_by_filter()`, y `count()` acepten filtros con operadores (Eq, Neq, Gt, Lt, Gte, Lte). Actualmente el SDK solo soporta Eq exact-match via `VantaMemoryMetadata`.

**Relación con hallazgo:** 🔴 Hallazgo 5 — El engine (IQL) soporta 6 operadores. El SDK (SDK `matches_memory_filters`) solo soporta `Eq`.

**Especificación:**

```rust
impl VantaEmbedded {
    // Extender list() para aceptar filtros avanzados
    pub fn list(
        &self,
        namespace: &str,
        options: VantaMemoryListOptions,
    ) -> Result<VantaMemoryListPage>;
}

// Extender VantaMemoryListOptions
pub struct VantaMemoryListOptions {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub filters: Option<VantaMemoryFilter>,  // NUEVO: reemplaza el viejo VantaMemoryMetadata
    pub include_payload: bool,
    pub include_metadata: bool,
}
```

**⚠️ Transición backward-compatible:**
- Mantener `filters: VantaMemoryMetadata` como `#[deprecated]`
- Agregar `filter_ops: Option<VantaMemoryFilter>` como nuevo campo
- En `list()`, si `filter_ops` está presente, usarlo; si no, caer a `filters` legacy

**Implementación del evaluador de filtros:**
- En `src/sdk/serialization/mod.rs`, extender `matches_memory_filters()` para aceptar `VantaMemoryFilter` con operadores
- Reusar la lógica de `compare_field_values()` de `src/physical_plan.rs` (líneas 142-189) o duplicar la lógica en el SDK
- **Decisión:** Duplicar lógica en SDK (evitar dependencia del SDK en physical_plan). La lógica es simple: match sobre tipos y comparar.

**Acceptance Criteria:**
- [ ] `VantaMemoryListOptions.filter_ops: Option<VantaMemoryFilter>` agregado
- [ ] `matches_memory_filters()` extendido para soportar 6 operadores
- [ ] `list()` usa nuevos filtros cuando están presentes
- [ ] `VantaMemoryMetadata` legacy mantenido como deprecated
- [ ] Tests: (a) Eq funciona, (b) Neq funciona, (c) Gt/Lt numérico, (d) filtros combinados AND, (e) compatibilidad legacy
- [ ] `cargo check -p vantadb` pasa

**Archivos tocados:**
- `src/sdk/types.rs` — `VantaMemoryListOptions` extendido
- `src/sdk/serialization/mod.rs` — `matches_memory_filters()` extendido con operadores
- `src/sdk/api.rs` — `list()` actualizado para pasar los nuevos filtros
- `tests/memory/filter_tests.rs` — nuevos tests

**Verificación:**
- [ ] `cargo check -p vantadb`
- [ ] `cargo clippy -p vantadb --deny warnings`
- [ ] `cargo nextest run --profile audit -p vantadb`
- [ ] Test manual: list con `--filter '{"age": {"$gt": 25}}'` (formato JSON por definir)

**Riesgos:**
- ⚠️ **Backward compatibility:** Cambiar `filters` de `VantaMemoryMetadata` a `Option<VantaMemoryFilter>` rompe API.
  - **Mitigación:** Campo nuevo + campo legacy deprecated.
- ⚠️ **Formato de filtro en CLI:** El formato JSON de filtros para CLI debe definirse.
  - **Decisión:** Usar `{"field": {"$gt": 25}}` formato MongoDB-like.
- ⚠️ **Coherencia con IQL:** Los operadores deben comportarse igual que en IQL.
  - **Mitigación:** Revisar `physical_plan.rs:compare_field_values()` como referencia de oro.

**Dependencias:** REC-001 (VantaMemoryFilter types)

**Esfuerzo:** 🟡 Medium (~2-3d)

---

### REC-007: [CLI] WAL compaction + vacuum CLI commands

**Descripción:** Agregar subcomandos CLI `vanta-cli wal compact` y `vanta-cli wal vacuum`. Ambos ya existen como métodos en el SDK (`VantaEmbedded::compact_wal()`, `VantaEmbedded::vacuum()`) pero no están expuestos en CLI.

**Relación con hallazgo:** 🟠 Hallazgo 6 — `compact_wal()` y `vacuum()` existen en engine y SDK. Solo falta binding CLI.

**Especificación:**

```
vanta-cli wal compact    # Llama a VantaEmbedded::compact_wal()
vanta-cli wal vacuum     # Llama a VantaEmbedded::vacuum()
```

**Acceptance Criteria:**
- [ ] `vanta-cli wal compact` implementado y funcional
- [ ] `vanta-cli wal vacuum` implementado y funcional
- [ ] Ambos commands abren DB, ejecutan operación, cierran DB
- [ ] `--help` documenta ambos subcomandos
- [ ] Tests CLI: test de integración para "no crash" (no necesita verificar efecto interno)

**Archivos tocados:**
- `src/cli.rs` — nuevo subcomando `Wal { Compact | Vacuum }` variant
- `src/cli_handlers/mod.rs` — nuevo módulo `wal.rs` o agregar a `db.rs`

**⚠️ Decisión de estructura:** ¿Nuevo archivo `wal.rs` en cli_handlers o agregar a `db.rs`?
- **Decisión:** Crear `src/cli_handlers/wal.rs` — operaciones de mantenimiento WAL merecen su propio módulo.
- Agregar `pub mod wal;` en `cli_handlers/mod.rs`

**Verificación:**
- [ ] `cargo check -p vantadb --features cli`
- [ ] `cargo clippy -p vantadb --deny warnings`
- [ ] Test manual: `vanta-cli wal compact` y `vanta-cli wal vacuum`
- [ ] Verificar que `vanta-cli wal --help` muestra ambos subcomandos

**Riesgos:**
- Bajo — es binding directo de funciones existentes. Sin lógica nueva.
- ⚠️ `vacuum()` puede tardar en DBs grandes. No hay indicador de progreso.
  - **Mitigación:** Agregar mensaje "Vacuuming..." al inicio y "Done." al final.

**Dependencias:** Ninguna

**Esfuerzo:** 🟢 Trivial (~1-2h)

---

### Checkpoint Fase 2: Engine Bindings

- [x] `cargo check -p vantadb --features cli` sin errores — **✅ REC-007**
- [x] `cargo clippy -p vantadb --deny warnings` sin nuevos warnings — **✅ REC-007**
- [ ] `cargo nextest run --profile audit -p vantadb` todos pasan
- [x] CLI nuevo commands aparecen en `--help` — **✅ REC-007**
- [ ] Multi-namespace search funciona en test manual
- [ ] Filtros avanzados funcionan con `--filter '{"field": {"$gt": 25}}'`

---

## Fase 3: 🟡 Features Complejas (P2 — Media, Requiere Diseño)

### REC-008: [Plan] Diseñar incremental backup + PITR CLI

**Descripción:** No implementar aún. Solo diseñar la arquitectura para incremental backup usando el WAL archiver existente como base. El objetivo es:

1. **Snapshot base:** Full backup inicial (ya existe)
2. **Incrementales:** Solo archivos WAL desde el último snapshot
3. **Restore:** Aplicar snapshot base + WALs incrementales
4. **CLI:** `vanta-cli backup --incremental`, `vanta-cli pitr restore --timestamp <ts>`

**Relación con hallazgo:** 🟠 Hallazgos 7 y 12 — Incremental backup no existe. PITR implementado pero sin CLI. WAL archiver existe como base.

**Lo que ya existe (reusable):**
- `WalArchiver` en `src/wal_archiver.rs` — archiva segmentos WAL rotados
- `PitrRestorer` en `src/wal_archiver.rs` — restore point-in-time
- Full backup en `src/cli_handlers/backup.rs`
- `VantaEmbedded::create_snapshot()` — instant snapshot

**Entregable de esta tarea:**
- Documento de diseño en `docs/architecture/incremental-backup.md`
- ADR en `docs/architecture/adr/`
- NO código — solo plan de implementación

**Acceptance Criteria:**
- [ ] Documento de diseño describe: snapshot strategy, WAL replay, retention, restore flow
- [ ] ADR captura decisiones: ¿snapshot completo o diferencial? ¿formato de backup? ¿cómo manejar WAL shipping?
- [ ] Plan de implementación con tareas detalladas y estimaciones
- [ ] Revisión de riesgos de data loss y recovery point objectives

**Archivos tocados (solo diseño):**
- `docs/architecture/incremental-backup.md`
- `docs/architecture/adr/NNN-incremental-backup.md`

**Dependencias:** Ninguna

**Esfuerzo:** 🟡 Medium (~1d de diseño)

---

### REC-009: [Plan] Analizar viabilidad de PQ (Product Quantization)

**Descripción:** Investigar si implementar Product Quantization 96x es viable y deseable. Actualmente hay ponytail note explícita: *"Simplified SQ8 only — no anisotropic quantization, no PQ, no GPU."*

**Relación con hallazgo:** 🟠 Hallazgo 8 — SQ8 + SCANN existen. PQ no. La nota ponytail fue una decisión consciente de simplificación.

**Preguntas a responder:**
1. ¿Cuál es el caso de uso que justifica PQ sobre el stack actual (SQ8 + SCANN)?
2. ¿Cuánto espacio de almacenamiento ahorraría PQ 96x vs SQ8?
3. ¿Cuál es el impacto en recall vs SQ8 + re-rank?
4. ¿Qué cambios de arquitectura requiere? (nuevo index file, nuevo algoritmo de distancia)
5. ¿Es compatible con los packeod offsets y el sistema LSM actual?

**Entregable:** Documento de análisis en `docs/research/pq-feasibility.md`

**Acceptance Criteria:**
- [ ] Análisis de tradeoffs completado
- [ ] Recomendación: implementar o no implementar, con justificación
- [ ] Si se recomienda implementar: plan de implementación con tareas
- [ ] Costo estimado en esfuerzo

**Dependencias:** Ninguna

**Esfuerzo:** 🟢 Small (~2-4h de investigación)

---

### Checkpoint Fase 3: Features Complejas

- [ ] `docs/architecture/incremental-backup.md` creado
- [ ] ADR de backup firmado
- [ ] `docs/research/pq-feasibility.md` creado
- [ ] Revisión humana de ambos documentos

---

## Fase 4: 🟢 Fixes Triviales (P3 — Quick Wins)

### REC-010: [Python] Agregar `py.typed` marker + configurar inclusión en wheel

**Descripción:** Agregar el archivo `py.typed` a `vantadb-python/` y configurar `[tool.maturin]` en `pyproject.toml` para que incluya los `.pyi` stubs en el wheel publicado. Esto habilita PEP 561 compliance.

**Relación con hallazgo:** 🟢 Hallazgo 12 — 5 `.pyi` existen pero `py.typed` no. Mypy/Pyright no pueden descubrir los stubs automáticamente.

**Especificación de implementación:**

1. **Crear `vantadb-python/vantadb_py/py.typed`:**
   - Archivo vacío (contenido: nada, o una línea `# PEP 561 marker`)
   - PEP 561 requiere que `py.typed` sea un archivo vacío o casi vacío colocado junto al paquete

2. **Configurar `pyproject.toml` en `vantadb-python/pyproject.toml`:**
   ```toml
   [tool.maturin]
   features = ["python_sdk"]
   module-name = "vantadb_py._vantadb_py"
   
   # PEP 561: include py.typed marker and .pyi stubs in wheel
   include = [
     { path = "vantadb_py/py.typed" },
     { path = "vantadb_py/*.pyi" },
   ]
   ```

3. **Verificar la inclusión en el wheel:**
   ```bash
   cd vantadb-python
   maturin build --release
   python -c "import zipfile; z = zipfile.ZipFile('target/wheels/*.whl'); print([n for n in z.namelist() if 'py.typed' in n or n.endswith('.pyi')])"
   ```

**⚠️ Verificación post-build:**
- El wheel debe contener `vantadb_py/py.typed`
- El wheel debe contener `vantadb_py/__init__.pyi` y `vantadb_py/vantadb_py.pyi`
- `mypy --strict -c "import vantadb_py"` no debe tener errores de "library stubs not found"

**Acceptance Criteria:**
- [ ] `vantadb-python/vantadb_py/py.typed` creado (archivo vacío)
- [ ] `pyproject.toml` configurado con `include` para stubs
- [ ] Build de wheel exitoso con `maturin build`
- [ ] Wheel contiene `py.typed` y todos los `.pyi`
- [ ] `mypy --strict` puede importar `vantadb_py` sin errores de stubs
- [ ] `cargo check -p vantadb-python` no se rompe (maturin build)

**Archivos tocados:**
- `vantadb-python/vantadb_py/py.typed` — NUEVO (archivo vacío)
- `vantadb-python/pyproject.toml` — agregar `[tool.maturin] include`

**Verificación:**
- [ ] `cd vantadb-python && maturin build`
- [ ] Inspeccionar wheel: `python -c "import zipfile; ..."`
- [ ] `mypy --strict vantadb-python/tests/` (si hay tests de tipo)

**Riesgos:** 
- ⚠️ **Maturin puede ignorar includes si no están en el formato correcto.**
  - **Mitigación:** Verificar versión de maturin y formato exacto de `include` en documentación oficial.
  - **Acción:** `webfetch` la documentación de `pyproject.toml` de maturin para confirmar sintaxis.
- ⚠️ **Provider packages (openai/ollama/litellm) también tienen .pyi.** Decidir si también reciben `py.typed` o se dejan para después.
  - **Decisión:** Solo core `vantadb-python` ahora. Providers en tarea separada si es necesario.

**Dependencias:** Ninguna

**Esfuerzo:** 🟢 Trivial (~30min-1h)

---

### Checkpoint Fase 4: Fixes Triviales

- [ ] `py.typed` existe en el paquete
- [ ] Wheel validado con py.typed + .pyi adentro
- [ ] mypy no se queja de stubs faltantes

---

## Fase 5: 📋 Plan-only / Futuro

### REC-999: [Meta] Corrección de `docs/progreso/README.md`

**Descripción:** Actualizar `docs/progreso/README.md` para corregir el estado de las tasks que estaban marcadas como completadas pero en realidad no existen o fueron eliminadas. Tasks específicas:

- Tasks de `delete_by_filter`, `count`, `similar_to_key`: Pasar de ✅ a ⚠️ (No implementado)
- Task de multi-namespace search: Pasar de ✅ a 🔴 (Nunca implementado)
- Task de py.typed: Pasar de ✅ a ⚠️ (Parcial — stubs existen, marker no)

**Acceptance Criteria:**
- [ ] `docs/progreso/README.md` refleja estado real después de investigación
- [ ] Cada cambio tiene referencia al hallazgo de la auditoría
- [ ] NO modificar `docs/Backlog.md` — solo progreso

**Archivos tocados:**
- `docs/progreso/README.md`

**Dependencias:** Investigación completa (este plan)

**Esfuerzo:** 🟢 Trivial (~30min)

---

## Tabla de Resumen de Tareas

| ID | Tarea | Prioridad | Esfuerzo | Dependencias | Fase |
|----|-------|-----------|----------|-------------|------|
| REC-001 | Definir VantaFilterOp + VantaMemoryFilter types | 🔴 P0 | 🟢 2-3h | Ninguna | F1 |
| REC-002 | Implementar delete_by_filter() SDK + CLI | 🔴 P0 | 🟡 1-2d | REC-001 | F1 |
| REC-003 | Implementar count() SDK + CLI | 🔴 P0 | 🟡 1d | REC-001 (opcional) | F1 |
| REC-004 | Implementar similar_to_key() SDK + CLI | 🔴 P0 | 🟡 1-2d | Ninguna | F1 |
| REC-005 | Multi-namespace search (search_multi/search_all) | 🟠 P1 | 🟡 4-7d | Ninguna | F2 |
| REC-006 | SDK metadata filters operadores | 🟠 P1 | 🟡 2-3d | REC-001 | F2 |
| ~~REC-007~~ | ~~WAL compact + vacuum CLI~~ | ~~🟠 P1~~ | ~~🟢 1-2h~~ | Ninguna | F2 **✅** |
| REC-008 | Diseñar incremental backup + PITR CLI | 🟡 P2 | 🟡 1d | Ninguna | F3 |
| REC-009 | Analizar viabilidad PQ (Product Quantization) | 🟡 P2 | 🟢 2-4h | Ninguna | F3 |
| ~~REC-010~~ | ~~py.typed marker + wheel inclusion~~ | 🟢 P3 | ~~🟢 30min~~ | Ninguna | F4 **✅** |
| REC-999 | Corrección progreso/README.md | 🟢 P3 | 🟢 30min | Investigación | F5 |

---

## Riesgos Globales y Mitigaciones

| Riesgo | Impacto | Probabilidad | Mitigación |
|--------|---------|-------------|------------|
| **Breaking changes en API pública** | Alto (afecta consumidores) | Media | REC-001/006 diseñados con backward compatibility explícita |
| **Performance de scan completo** | Medio (DBs grandes lentas) | Alta para DBs >1M records | Documentar límites, planificar optimización V2 |
| **Duplicación lógica de filtros** | Medio (código extra) | Alta | Acceptado deliberadamente para mantener SDK independiente de IQL |
| **Merge de scores multi-namespace** | Medio (resultados inconsistentes) | Media | Investigar normalización de scores antes de implementar REC-005 |
| **maturin include syntax** | Bajo (build falla) | Baja | Verificar documentación oficial de maturin |
| **PQ decide no implementarse** | Bajo (se aclara scope) | Alta | Ponytail note existente es válida. PQ es feature request, no deuda |

---

## Open Questions

1. **Formato JSON de filtros para CLI:** ¿MongoDB-like (`{"field": {"$gt": 25}}`) o formato plano (`field:op:value`)?
   - Pendiente de decisión del usuario.

2. **¿`count()` debe contar TTL-expired records también?**
   - `purge_expired()` existe separadamente. `count()` debe contar solo registros vivos.
   - Confirmar con engine team.

3. **`similar_to_key()` — ¿incluir o excluir el mismo registro?**
   - Decisión tentativa: excluir. Confirmar preferencia del usuario.

4. **Multi-namespace search — ¿Qué hacer con `VantaMemorySearchRequest` que tiene `namespace: String`?**
   - Opciones: (a) ignorar el campo si se usa `search_multi()`, (b) agregar `namespaces: Vec<String>` al request.
   - Decisión tentativa: Agregar `namespaces: Vec<String>` a `VantaMemorySearchRequest`.

5. **¿Incremental backup debe ser feature-gated?**
   - `WalArchiver` ya está tras feature flag `pitr`. ¿Mantener misma estrategia?

---

## Appendix: Referencias de Código

### Archivos clave identificados en la investigación

| Archivo | Propósito | Líneas relevantes |
|---------|-----------|-------------------|
| `src/sdk/api.rs` | Métodos públicos del SDK | Todo el archivo (~700L) |
| `src/sdk/types.rs` | Tipos del SDK (VantaMemory*, VantaValue, etc.) | ~500L |
| `src/sdk/serialization/mod.rs` | `matches_memory_filters()` — solo Eq | Línea 368 |
| `src/sdk/search/mod.rs` | FilterStrategy (PreFilter/InFilter/PostFilter) | ~2000L |
| `src/query.rs` | `RelOp` enum — 6 operadores | Líneas 129-143 |
| `src/physical_plan.rs` | `evaluate_condition()` — implementación de operadores | Líneas 142-189 |
| `src/cli.rs` | Commands enum del CLI | ~307L |
| `src/cli_handlers/crud.rs` | CRUD CLI handlers | ~(existente) |
| `src/cli_handlers/search.rs` | Search CLI handlers | ~(existente) |
| `src/wal_archiver.rs` | WalArchiver + PitrRestorer | ~417L |
| `src/storage/engine/maintenance.rs` | `compact_wal()` + `vacuum()` impl | Líneas 123, 626 |
| `vantadb-python/pyproject.toml` | Build config maturin | ~(existente) |
| `vantadb-python/vantadb_py/__init__.pyi` | Python type stubs core | ~284L |
| `docs/progreso/README.md` | Bitácora de progreso (contiene estado incorrecto) | ~(existente) |
| `src/storage/engine/mod.rs` | StorageEngine, PipelineMode | ~537L |

### Commits de referencia en git history

| Commit | Relevancia |
|--------|-----------|
| `e9371ea8` | AUD-09: Eliminó `cmd_delete_by_filter`, `cmd_count`, `cmd_search_similar` (CLI handlers, ~560 LOC) |
| `72d334c3` | Refactor masivo: sdk.rs (4230L) → 7 sub-módulos. SDK nunca tuvo estas funciones. |
| `f55750cc` | Contiene referencias a `delete_by_filter` en fix de backup |

---

## Criterios de Salida del Plan

- [ ] REC-001 a REC-004 implementados y verificados (SDK functions core)
- [x] REC-005 a REC-007 implementados y verificados (engine bindings) — **✅ 2026-07-29 (REC-007)**
- [ ] REC-008 y REC-009 diseñados y documentados (features complejas)
- [x] REC-010 implementado y verificado (Python PEP 561) — **✅ 2026-07-29**
- [ ] REC-999: progreso/README.md refleja estado real
- [ ] `just verify` pasa completo
- [ ] Changelog actualizado con cambios visibles al usuario
- [ ] Push a develop, PR a main
