# AUDITORÍA COMPLETA — VantaDB

| Campo | Detalle |
|---|---|
| **Repositorio** | [github.com/DevpNess/Vantadb](https://github.com/DevpNess/Vantadb) |
| **Rama analizada** | `develop` (commit `63b0101d`) |
| **Tipo de proyecto** | Base de datos vectorial embebida escrita en Rust |
| **Fecha de auditoría** | 2025-07-27 |
| **Alcance** | Análisis estático completo del código fuente |

---

## RESUMEN EJECUTIVO

VantaDB es una base de datos vectorial embebida con indexación HNSW, WAL, almacenamiento memory-mapped, un lenguaje de consulta propio (IQL), y sub-proyectos complementarios (web Next.js, SDK TypeScript, SDK Python, servidor MCP, integraciones con frameworks de IA).

Se ejecutaron **6 sub-agentes especializados** que analizaron de forma paralela la estructura del proyecto, dependencias, seguridad, lógica de código Rust, algoritmos, WAL, almacenamiento, web frontend, SDKs, integraciones y pruebas.

| Severidad | Total | Impacto Principal |
|-----------|-------|-------------------|
| CRÍTICO | **11** | Pérdida de datos, panic en producción, builds rotos |
| ALTO | **15** | Inconsistencias lógicas, seguridad, datos corruptos |
| MEDIO | **26** | Configuración, rendimiento, calidad de código |
| BAJO | **15** | Código muerto, nombres confusos, limpieza |
| **TOTAL** | **~67 únicos** | Documentados con archivo, línea y código |

> **Hallazgo principal:** El Dockerfile está completamente roto (8 directorios inexistentes + versión de Rust incorrecta), el WAL tiene una brecha de durabilidad por defecto que puede perder datos en crash, y el índice IVF se vuelve obsoleto silenciosamente después de la primera búsqueda.

---

## ESTRUCTURA DEL PROYECTO ANALIZADA

```
vantadb-audit/
├── src/                    # Core Rust library (~20k+ líneas)
│   ├── engine.rs           # InMemoryEngine (943 líneas)
│   ├── wal.rs              # WAL single-shard (977 líneas)
│   ├── wal_sharded.rs      # WAL multi-shard (562 líneas)
│   ├── wal_archiver.rs     # Archivado y PITR (417 líneas)
│   ├── wal_shipping.rs     # Replicación WAL (290 líneas)
│   ├── cli_server.rs       # Servidor HTTP (axum)
│   ├── cli_handlers/       # Handlers HTTP (backup, etc.)
│   ├── config.rs           # Configuración
│   ├── crypto.rs           # Cifrado AES-256-GCM
│   ├── rbac.rs             # Control de acceso
│   ├── parser/mod.rs       # Parser IQL (nom)
│   ├── index/              # HNSW, IVF, distancias SIMD
│   │   ├── graph.rs        # Grafo HNSW
│   │   ├── search.rs       # Búsqueda (HNSW + IVF + brute-force)
│   │   ├── distance.rs     # Cálculos de distancia (f32, SIMD, SQ8)
│   │   └── ivf.rs          # Índice IVF
│   ├── storage/            # Capa de almacenamiento
│   │   ├── vfile.rs        # Memory-mapped files
│   │   ├── ops.rs          # Operaciones de almacenamiento
│   │   ├── archive.rs      # Compacción de archivos
│   │   ├── engine/         # StorageEngine completo
│   │   └── wal.rs          # WAL para StorageEngine
│   ├── sdk/                # SDK Rust (api.rs, search/)
│   └── node.rs             # Estructura de nodos
├── web/                    # Next.js 16 + shadcn/ui frontend
├── vantadb-ts/             # TypeScript SDK (WASM)
├── vantadb-python/         # Python SDK (PyO3)
├── vantadb-mcp/            # MCP Server
├── vantadb-server/         # Server wrapper
├── vantadb-wasm/           # WASM build
├── providers/              # Rust providers (openai, ollama, litellm)
├── integrations/           # Python integrations (mem0, letta, crewai, etc.)
├── fuzz/                   # Fuzz targets (4)
├── tests/                  # Suite de tests
├── Cargo.toml              # Workspace configuration
├── Dockerfile              # Docker build
├── deny.toml               # Cargo-deny security config
└── Justfile                # Task runner (PowerShell)
```

---

## 1. ERRORES CRÍTICOS

> **Impacto:** Pérdida de datos, panic en producción, builds completamente rotos.

---

### CRIT-01: ShardedWal::recover usa semántica de checkpoint incorrecta — PÉRDIDA DE DATOS

| Campo | Detalle |
|---|---|
| **Archivo** | `src/wal_sharded.rs:110-117` |
| **Severidad** | CRÍTICO |
| **Categoría** | WAL / Durabilidad |

**Descripción:**

El `checkpoint_seq` es GLOBAL (suma de todos los shards), pero `current_seq` se reinicia a `0` por cada shard durante la recuperación.

```rust
let mut current_seq = 0u64;
while let Some(record) = reader.next_record()? {
    current_seq += 1;
    if current_seq <= checkpoint_seq {
        continue;   // <- seq local por shard, pero checkpoint_seq es GLOBAL
    }
    f(record)?;
}
```

**Impacto:** Con N=4 shards y `checkpoint_seq=100`, cada shard salta sus primeros 100 registros — se saltan **400 registros** en vez de 100. La recuperación de WAL es silenciosamente incorrecta en cualquier configuración multi-shard.

**Recomendación:** Portar la matemática round-robin de `src/storage/engine/init.rs:386-419` o deprecar el método.

---

### CRIT-02: compact_layout panic al leer datos truncados

| Campo | Detalle |
|---|---|
| **Archivo** | `src/storage/archive.rs:111-116` |
| **Severidad** | CRÍTICO |
| **Categoría** | Storage / Panic en producción |

**Descripción:**

`copy_from_slice` requiere longitudes iguales. Si el archivo fuente está truncado, la validación con `.min(old_data.len())` produce un destino más largo que el origen:

```rust
let copy_len = (header_size + vec_size_aligned) as usize;
tmp_mmap[write_cursor as usize..(write_cursor as usize + copy_len)]
    .copy_from_slice(&old_data[src_start..src_end.min(old_data.len())]);
//                    ^^^^^^^ longitud variable     ^^^^^^^^^^^^^^^^^^^^^^^ longitud fija
```

**Impacto:** Panic fatal del proceso durante compactación si el vstore tiene un header cuyo `vector_len` reclama más bytes de los que realmente tiene (ej: crash mid-write, truncamiento parcial).

**Recomendación:** Validar que `old_data.len() - src_start >= copy_len` antes de `copy_from_slice`; retornar error si no.

---

### CRIT-03: .expect() panic en deserialización de claves

| Campo | Detalle |
|---|---|
| **Archivo** | `src/storage/engine/ops.rs:1504-1506` |
| **Severidad** | CRÍTICO |
| **Categoría** | Engine / Panic en producción |

**Descripción:**

```rust
let id = u128::from_le_bytes(
    key.as_slice().try_into().expect("key slice fits [u8; 16]"),
);
```

Aunque existe un chequeo de longitud en la línea 1500, `.expect()` panica en producción si el backend devuelve datos corruptos.

**Recomendación:** Usar `?` con un `VantaError` apropiado.

---

### CRIT-04: Errores de tombstone silenciosamente tragados — REGISTROS FANTASMA

| Campo | Detalle |
|---|---|
| **Archivo** | `src/storage/engine/ops.rs:371, 653, 845` |
| **Severidad** | CRÍTICO |
| **Categoría** | Storage / Consistencia |

**Descripción:**

Cuando la escritura KV falla después de escribir en vstore, se intenta crear un tombstone. Si ese también falla, el error se descarta:

```rust
if let Err(e) = self.backend.put(...) {
    if let Some(mut hdr) = vstore.read_header(offset) {
        hdr.flags |= FLAG_TOMBSTONE;
        let _ = vstore.write_header(offset, &hdr);   // <- ERROR TRAGADO
    }
    return Err(e);
}
```

**Impacto:** Queda un registro "zombie" vivo sin metadatos KV. En recuperación, `rebuild_hnsw_from_vstore` lo reindexa como nodo vivo (solo salta registros con `FLAG_TOMBSTONE` set).

**Recomendación:** Agregar `tracing::error!` y ya sea reintentar o panic con mensaje claro.

---

### CRIT-05: compact_layout traga error de flush + sin sync_all antes de rename

| Campo | Detalle |
|---|---|
| **Archivo** | `src/storage/archive.rs:99` |
| **Severidad** | CRÍTICO |
| **Categoría** | Storage / Durabilidad |

**Descripción:**

```rust
if end > new_file_size {
    let _ = tmp_mmap.flush();   // <- ERROR TRAGADO
    drop(tmp_mmap);
    tmp_file.set_len(end + 4096).map_err(VantaError::IoError)?;
    tmp_mmap = unsafe { MmapOptions::new().map_mut(&tmp_file)... };
}
```

Si `tmp_mmap.flush()` falla (ej: disco lleno), el código continúa. No hay `tmp_file.sync_all()` antes del rename final.

**Impacto:** Datos sucios no flushed pueden ser renombrados como archivo válido. Un crash después del rename deja un archivo parcialmente escrito.

**Recomendación:** Propagar el error de `flush()` y agregar `sync_all()` antes del rename.

---

### CRIT-06: WAL no sincroniza a disco por defecto — PÉRDIDA DE DATOS EN CRASH

| Campo | Detalle |
|---|---|
| **Archivo** | `src/wal.rs:321-330`, `src/engine.rs:144`, `src/config.rs:150` |
| **Severidad** | CRÍTICO |
| **Categoría** | WAL / Durabilidad |

**Descripción:**

`InMemoryEngine::with_wal` usa `SyncMode::Periodic` con `flush_threshold = None`:

```rust
// src/engine.rs:144
let sharded = ShardedWal::new(&path, 4, crate::config::SyncMode::Periodic)?;
```

```rust
// src/wal.rs:321-330
fn maybe_sync(&mut self) -> Result<()> {
    if self.sync_mode == crate::config::SyncMode::Always {
        self.sync()?;
    } else if let Some(threshold) = self.flush_threshold {
        if self.records_since_sync >= threshold as u64 {
            self.sync()?;
        }
    }
    Ok(()) // <- NO-OP cuando flush_threshold es None
}
```

**Impacto:** Los registros WAL van solo al buffer en memoria (64 KB BufWriter). Un crash del proceso puede perder hasta 64 KB de registros WAL confirmados.

**Recomendación:** Establecer un `flush_threshold` por defecto razonable (ej: 1000 registros o 1 MB) cuando `SyncMode::Periodic`.

---

### CRIT-07: Dockerfile referencia 8 directorios inexistentes — BUILD ROTO

| Campo | Detalle |
|---|---|
| **Archivo** | `Dockerfile:32-39, 45` |
| **Severidad** | CRÍTICO |
| **Categoría** | Build / Infraestructura |

**Descripción:**

```dockerfile
COPY vantadb-mem0/Cargo.toml vantadb-mem0/        # <- NO EXISTE
COPY vantadb-letta/Cargo.toml vantadb-letta/       # <- NO EXISTE
COPY vantadb-crewai/Cargo.toml vantadb-crewai/     # <- NO EXISTE
COPY vantadb-dspy/Cargo.toml vantadb-dspy/         # <- NO EXISTE
COPY vantadb-haystack/Cargo.toml vantadb-haystack/ # <- NO EXISTE
COPY vantadb-litellm/Cargo.toml vantadb-litellm/   # <- NO EXISTE
COPY vantadb-openai/Cargo.toml vantadb-openai/     # <- NO EXISTE
COPY vantadb-ollama/Cargo.toml vantadb-ollama/     # <- NO EXISTE
```

Ninguno de estos directorios existe. Los Rust providers están en `providers/{openai,ollama,litellm}/` y las integraciones Python están en `integrations/{mem0,letta,crewai,dspy,haystack,...}/` (sin `Cargo.toml`).

**Impacto:** `docker build` falla inmediatamente en la línea 32 con `COPY failed: file not found`.

**Recomendación:** Eliminar las líneas 32-39 y el loop en la línea 45, o reescribir para referenciar las rutas reales.

---

### CRIT-08: Versión de Rust en Docker por debajo del MSRV

| Campo | Detalle |
|---|---|
| **Archivo** | `Dockerfile:4` vs `Cargo.toml:5, 577` |
| **Severidad** | CRÍTICO |
| **Categoría** | Build / Infraestructura |

**Descripción:**

| Fuente | Versión de Rust |
|---|---|
| `Dockerfile` línea 4 | `ARG RUST_VERSION=1.94.0` |
| `Cargo.toml` líneas 5, 577 | `rust-version = "1.94.1"` |

**Impacto:** El build dentro del contenedor falla con `rustc 1.94.0 is not supported by this package`.

**Recomendación:** Cambiar a `ARG RUST_VERSION=1.94.1` (o latest stable).

---

### CRIT-09: Providers usan herencia de workspace sin ser miembros

| Campo | Detalle |
|---|---|
| **Archivo** | `providers/{openai,ollama,litellm}/Cargo.toml:3-5, 23` vs `Cargo.toml:556-562` |
| **Severidad** | CRÍTICO |
| **Categoría** | Build / Configuración |

**Descripción:**

Los tres providers usan `version.workspace = true`, `edition.workspace = true`, `rust-version.workspace = true` y `[lints] workspace = true`, pero no están ni en `workspace.members` ni en `workspace.exclude`.

**Impacto:** Per Cargo's workspace inheritance rules, los crates que usan `*.workspace = true` DEBEN ser miembros del workspace (o estar en `exclude`). Los providers no se pueden compilar desde la raíz del workspace.

**Recomendación:** Agregar los providers a `workspace.exclude`, o agregar un bloque `[workspace]` vacío a cada `Cargo.toml` de provider (como hace `fuzz/Cargo.toml`).

---

### CRIT-10: Dependencias opcionales prometheus y rayon sin feature — ~440 LÍNEAS DE CÓDIGO MUERTO

| Campo | Detalle |
|---|---|
| **Archivo** | `Cargo.toml:51, 75` + `src/metrics/core/registry.rs` + `src/sdk/api.rs` |
| **Severidad** | CRÍTICO |
| **Categoría** | Build / Código muerto |

**Descripción:**

```toml
# Cargo.toml
prometheus = { version = "0.14", optional = true }  # línea 51
rayon = { version = "1.12", optional = true }          # línea 75
```

La tabla `[features]` (líneas 91-127) **no define ningún feature `prometheus` o `rayon`**. Sin embargo, el código tiene:
- **49 bloques `#[cfg(feature = "prometheus")]`** en `src/metrics/core/registry.rs`
- **7 bloques `#[cfg(feature = "prometheus")]`** en `src/metrics/core/mod.rs`
- **3 bloques `#[cfg(feature = "rayon")]`** en `src/sdk/api.rs:128, 141, 145`

**Impacto:** Toda la integración Prometheus (~440+ líneas) y el path paralelo con rayon son **permanentemente inalcanzables**.

**Recomendación:** Agregar a `[features]`:
```toml
prometheus = ["dep:prometheus"]
rayon = ["dep:rayon"]
```
O eliminar el código muerto y los deps opcionales si las integraciones fueron abandonadas.

---

### CRIT-11: WalArchiver::archive_segment colisión de timestamps y race condition

| Campo | Detalle |
|---|---|
| **Archivo** | `src/wal_archiver.rs:76-88` |
| **Severidad** | CRÍTICO |
| **Categoría** | WAL / Race condition |

**Descripción:**

```rust
let timestamp = web_time::SystemTime::now()
    .duration_since(web_time::UNIX_EPOCH)
    .unwrap_or_default()
    .as_millis();
let archive_name = format!("{}.{}", filename.to_string_lossy(), timestamp);
if dest.exists() {
    std::fs::remove_file(&dest)?;
}
std::fs::rename(source_path, &dest)?;
```

**Problemas:**
1. Precisión de milisegundos: dos segmentos archivados en <1ms colisionan
2. `remove_file` + `rename` no es atómico — llamadas concurrentes pueden intercalarse

**Impacto:** Pérdida de datos de archivo si dos segmentos se archivan concurrentemente.

**Recomendación:** Usar precisión de nanosegundos + sufijo UUID, o `tempfile` + atomic rename.

---

## 2. ERRORES ALTOS

> **Impacto:** Inconsistencias lógicas, brechas de seguridad, datos corruptos silenciosamente.

---

### ALTO-01: Parser trunca literales flotantes a enteros silenciosamente

| Campo | Detalle |
|---|---|
| **Archivo** | `src/parser/mod.rs:88-96` |
| **Severidad** | ALTO |
| **Categoría** | Parser / Corrupción de datos |

**Descripción:**

```rust
fn parse_literal_field_value(i: &str) -> IResult<&str, FieldValue> {
    alt((
        map(string_literal, FieldValue::String),
        map(ws(tag("true")), |_| FieldValue::Bool(true)),
        map(ws(tag("false")), |_| FieldValue::Bool(false)),
        map(ws(tag("null")), |_| FieldValue::Null),
        map(ws(parse_i64), FieldValue::Int),       // <- consume "3" de "3.14"
        map(ws(float), |f: f32| FieldValue::Float(f as f64)),  // <- nunca alcanzado
    ))(i)
}
```

`parse_i64` consume dígitos y se detiene en `.`. Como está **antes** que `float` en `alt()`, input como `3.14` se parsea como `Int(3)`, no `Float(3.14)`.

**Impacto:**
- `INSERT NODE#1 TYPE Item { price: 3.14 }` -> price almacenado como `Int(3)`
- `UPDATE NODE#5 SET score = 0.95` -> score almacenado como `Int(0)`

**Recomendación:** Intercambiar el orden para intentar `float` antes que `parse_i64`.

---

### ALTO-02: Índice IVF nunca se invalida después de inserts

| Campo | Detalle |
|---|---|
| **Archivo** | `src/index/search.rs:464-476` |
| **Severidad** | ALTO |
| **Categoría** | Index / Datos obsoletos |

**Descripción:**

```rust
if self.config.index_type == IndexType::Ivf {
    let mut guard = self.ivf_index.lock();
    if guard.is_none() {
        *guard = Some(crate::index::ivf::IvfIndex::build(&self.nodes, &ivf_config));
    }
    let ivf = guard.as_ref().unwrap();
    return ivf.search(query_vec, top_k, query_mask);
}
```

El IVF se construye lazy en la primera búsqueda y se cachea **para siempre**. Nodos insertados después son **invisibles** para búsquedas IVF. El método `add()` en `graph.rs` no limpia ni reconstruye el índice.

**Impacto:** Degradación silenciosa de recall. Queries pierden vectores recién insertados.

**Recomendación:** Invalidar/reconstruir el índice IVF en `CPIndex::add()` o documentar que IVF requiere rebuild manual.

---

### ALTO-03: Derivación de clave de cifrado débil (single SHA-256)

| Campo | Detalle |
|---|---|
| **Archivo** | `src/crypto.rs:96-103` |
| **Severidad** | ALTO |
| **Categoría** | Seguridad / Criptografía |

**Descripción:**

```rust
pub fn new(key: &[u8]) -> Self {
    let key_bytes = if key.len() == 32 {
        let mut k = [0u8; 32];
        k.copy_from_slice(key);
        k
    } else {
        Sha256::digest(key).into()  // Single SHA-256, sin key stretching
    };
```

Cuando la clave no tiene exactamente 32 bytes, un solo SHA-256 deriva la clave sin key stretching. Claves cortas pueden ser brute-forceadas instantáneamente.

**Recomendación:** Usar PBKDF2, Argon2id, o HKDF con salt aleatorio.

---

### ALTO-04: X-Forwarded-For confiado sin validación — IP Spoofing

| Campo | Detalle |
|---|---|
| **Archivo** | `src/cli_server.rs:229-238` |
| **Severidad** | ALTO |
| **Categoría** | Seguridad / Network |

**Descripción:**

```rust
pub fn client_ip(req: &axum::extract::Request) -> String {
    if let Some(forwarded) = req.headers().get("x-forwarded-for") {
        if let Ok(ip_str) = forwarded.to_str() {
            if let Some(ip) = ip_str.split(',').next().map(|s| s.trim()) {
                if !ip.is_empty() {
                    return ip.to_string();
                }
            }
        }
    }
```

Un atacante puede bypass el rate limiter de autenticación (5 intentos/60s) enviando diferentes valores de `X-Forwarded-For`.

**Recomendación:** Solo confiar en `X-Forwarded-For` cuando hay un proxy reverso configurado. Usar `ConnectInfo<SocketAddr>` como fuente autoritativa.

---

### ALTO-05: Sin límite de tamaño de body en endpoint de query

| Campo | Detalle |
|---|---|
| **Archivo** | `src/cli_server.rs:406` |
| **Severidad** | ALTO |
| **Categoría** | Seguridad / Network |

**Descripción:** El endpoint `/api/v2/query` acepta JSON sin límite explícito de tamaño.

**Recomendación:** Agregar `DefaultBodyLimit::max(1_000_000)` (1MB) al router.

---

### ALTO-06: Dev mode bypassa toda autenticación silenciosamente

| Campo | Detalle |
|---|---|
| **Archivo** | `src/cli_server.rs:268-270` |
| **Severidad** | ALTO |
| **Categoría** | Seguridad / Auth |

**Descripción:** Sin `VANTADB_API_KEY`, todas las requests pasan sin auth ni logging. `VANTADB_REQUIRE_AUTH` existe pero defaultea a `false`.

**Recomendación:** Loggear un warning por cada request no autenticada en dev mode.

---

### ALTO-07: Sin configuración CORS

| Campo | Detalle |
|---|---|
| **Archivo** | `src/cli_server.rs:~140-175` |
| **Severidad** | ALTO |
| **Categoría** | Seguridad / Network |

**Descripción:** No hay headers CORS configurados. El servidor no puede usarse desde aplicaciones web sin proxy reverso.

**Recomendación:** Agregar middleware CORS configurable.

---

### ALTO-08: ShardedWal::rotate_all — ventana de race condition

| Campo | Detalle |
|---|---|
| **Archivo** | `src/wal_sharded.rs:143-159` |
| **Severidad** | ALTO |
| **Categoría** | WAL / Concurrency |

**Descripción:**

```rust
for shard in &self.shards {
    let replacement = {
        let mut guard = shard.lock();
        guard.sync()?;
        WalWriter::open_with_buffer(&path, ...)?   // <- lock liberado aquí
    };
    *shard.lock() = replacement;                    // <- re-adquirido aquí
}
```

Entre la liberación del lock interno y la re-adquisición externa, un `append()` concurrente puede escribir al writer viejo que se descarta.

**Recomendación:** Mantener el lock a través de ambas operaciones.

---

### ALTO-09: InMemoryEngine::with_wal hardcodea 4 shards

| Campo | Detalle |
|---|---|
| **Archivo** | `src/engine.rs:144` |
| **Severidad** | ALTO |
| **Categoría** | WAL / Compatibilidad |

**Descripción:** Si la DB fue abierta previamente con diferente cantidad de shards, los archivos siguen naming diferente (`vanta.wal` vs `vanta.shard0.wal`). Recuperación con 4 shards busca archivos que no existen -> pérdida silenciosa.

**Recomendación:** Persistir el conteo de shards en metadatos.

---

### ALTO-10: WalShipper::run_loop sin shutdown ni backoff

| Campo | Detalle |
|---|---|
| **Archivo** | `src/wal_shipping.rs:132-144` |
| **Severidad** | ALTO |
| **Categoría** | WAL / Operaciones |

**Descripción:**
- Return type `!` — no hay forma de detener el hilo
- Mismo sleep en éxito y fallo persistente -> hammering cada 1s
- Default `replica_url` es vacía -> cada POST falla pero el loop nunca sale
- No hay shutdown token ni atomic flag

**Recomendación:** Agregar señal de shutdown + backoff exponencial.

---

### ALTO-11: save_vector_index falla en Windows

| Campo | Detalle |
|---|---|
| **Archivo** | `src/storage/engine/maintenance.rs:149-164` |
| **Severidad** | ALTO |
| **Categoría** | Storage / Cross-platform |

**Descripción:** En Windows, `std::fs::rename` falla si cualquier handle (incluyendo mmap) está abierto al archivo. El mmap se mueve a `new_index` antes del rename, así que el mapping sigue vivo.

**Recomendación:** Dropear el mmap antes del rename.

---

### ALTO-12: web/next.config.ts tiene ignoreBuildErrors: true

| Campo | Detalle |
|---|---|
| **Archivo** | `web/next.config.ts:7` |
| **Severidad** | ALTO |
| **Categoría** | Frontend / Build |

**Descripción:** Errores TypeScript son suprimidos en build time. Bugs de tipos pueden llegar a producción silenciosamente.

**Recomendación:** Cambiar a `ignoreBuildErrors: false` y corregir errores TypeScript resultantes.

---

### ALTO-13: Type guard isMemoryRecord() rechaza registros válidos

| Campo | Detalle |
|---|---|
| **Archivo** | `vantadb-ts/src/guards.ts:16-19` |
| **Severidad** | ALTO |
| **Categoría** | SDK TypeScript / Types |

**Descripción:** El guard verifica `typeof obj.version === "string"` pero el tipo `MemoryRecord` permite `string | number`. Registros donde WASM retorna números son falsamente rechazados.

**Recomendación:** Cambiar checks a `typeof obj.version === "string" || typeof obj.version === "number"`.

---

### ALTO-14: collection_stats y collection_list en MCP cargan TODOS los registros en memoria

| Campo | Detalle |
|---|---|
| **Archivo** | `vantadb-mcp/src/lib.rs:1327-1352, 1364-1368` |
| **Severidad** | ALTO |
| **Categoría** | MCP / OOM |

**Descripción:** Sin paginación ni guard de tamaño. Namespaces grandes causan OOM.

**Recomendación:** Agregar paginación y límite de tamaño.

---

### ALTO-15: Todas las 9 integraciones Python en versión 0.3.0

| Campo | Detalle |
|---|---|
| **Archivo** | `integrations/*/pyproject.toml` |
| **Severidad** | ALTO |
| **Categoría** | Integraciones / Versiones |

**Descripción:** Todas las 9 integraciones están en versión 0.3.0 mientras el core es 0.4.0. Además especifican `vantadb-py>=0.2` (bound demasiado loose).

**Recomendación:** Bump a 0.4.0 y pin `vantadb-py>=0.4.0,<0.5.0`.

---

## 3. ERRORES MEDIOS

> **Impacto:** Configuración incorrecta, rendimiento subóptimo, calidad de código.

---

### MED-01: exclude en tabla [workspace.package] incorrecta

| Campo | Detalle |
|---|---|
| **Archivo** | `Cargo.toml:574-578` |
| **Severidad** | MEDIO |
| **Categoría** | Configuración |

**Descripción:** `exclude = ["fuzz"]` está bajo `[workspace.package]` (tabla incorrecta). `exclude` es una clave `[workspace]`, no `[workspace.package]`. Es ignorada silenciosamente por Cargo.

**Recomendación:** Mover a la tabla `[workspace]`.

---

### MED-02: .gitignore ignora .env.example

| Campo | Detalle |
|---|---|
| **Archivo** | `.gitignore:6,67` |
| **Severidad** | MEDIO |
| **Categoría** | Configuración |

**Descripción:** La línea 6 ignora explícitamente `.env.example`. La línea 67 `.env.*` también lo atrapa. No hay negación `!.env.example` como sí existe para `.env.tokens.example`.

**Recomendación:** Agregar `!.env.example` después de línea 68.

---

### MED-03: Fórmula Homebrew con versión y SHA256 obsoletos

| Campo | Detalle |
|---|---|
| **Archivo** | `Formula/vantadb.rb:19, 30, 35, 43, 48` |
| **Severidad** | MEDIO |
| **Categoría** | Build / Release |

**Descripción:** Versión `0.2.0` vs workspace `0.4.0`. Los 4 valores SHA256 son `0000...0000` placeholders.

---

### MED-04: Justfile solo funciona con PowerShell

| Campo | Detalle |
|---|---|
| **Archivo** | `Justfile:5` |
| **Severidad** | MEDIO |
| **Categoría** | DX / Cross-platform |

**Descripción:** `set shell := ["pwsh", "-NoProfile", "-Command"]` — contribuidores en Linux/macOS sin PowerShell no pueden ejecutar `just`.

---

### MED-05: Vectores de norma cero silenciosamente descartados en Cosine

| Campo | Detalle |
|---|---|
| **Archivo** | `src/index/graph.rs:561-570` |
| **Severidad** | MEDIO |
| **Categoría** | Index / Lógica |

**Descripción:** Cuando se inserta un vector de norma cero con `DistanceMetric::Cosine`, el nodo se inserta y luego se remueve inmediatamente. `add()` retorna `()` — el caller cree que el insert tuvo éxito.

**Recomendación:** Retornar un error en vez de descartar silenciosamente.

---

### MED-06: euclidean_distance_sq_with_norms puede retornar negativo

| Campo | Detalle |
|---|---|
| **Archivo** | `src/index/distance.rs:341-349` |
| **Severidad** | MEDIO |
| **Categoría** | Index / Precisión numérica |

**Descripción:** `a_norm_sq + b_norm_sq - 2.0 * dot` puede ser ligeramente negativo por floating-point rounding para vectores idénticos. Esto corrompe la selección de vecinos HNSW donde no hay clamping.

---

### MED-07: NaN corrompe el heap del HNSW durante construcción

| Campo | Detalle |
|---|---|
| **Archivo** | `src/index/graph.rs:253-263, 278-288` |
| **Severidad** | MEDIO |
| **Categoría** | Index / Algoritmo |

**Descripción:** `NodeSim` mapea `NaN` a `Equal` en `cmp()`. Un nodo con similitud NaN nunca se evicciona del candidate set, corrompiendo la topología del grafo durante `insert_hnsw`.

---

### MED-08: Paginación basada en conteo pre-filtro

| Campo | Detalle |
|---|---|
| **Archivo** | `src/sdk/api.rs:282-335` |
| **Severidad** | MEDIO |
| **Categoría** | SDK / API |

**Descripción:** `next_cursor` se calcula de `unique_ids.len()` (pre-filtro), pero `records` puede tener menos items por filtros o TTL. El API reporta `next_cursor: Some(N)` cuando las páginas restantes pueden estar vacías -> loop infinito para el cliente.

---

### MED-09: frame_len en EncryptionStream sin límite

| Campo | Detalle |
|---|---|
| **Archivo** | `src/crypto.rs:236-237` |
| **Severidad** | MEDIO |
| **Categoría** | Seguridad / OOM |

**Descripción:** `let frame_len = u32::from_le_bytes(len_buf) as usize;` permite hasta 4GB de allocation. Agregar bound razonable (ej: 512MB).

---

### MED-10: Mensajes de panic filtrados a clientes HTTP

| Campo | Detalle |
|---|---|
| **Archivo** | `src/cli_server.rs:454` |
| **Severidad** | MEDIO |
| **Categoría** | Seguridad / Info leakage |

**Descripción:** `format!("Internal server error: execution task panicked: {}", e)` expone detalles internos al cliente.

---

### MED-11: Integer overflow en write_node_to_vstore

| Campo | Detalle |
|---|---|
| **Archivo** | `src/storage/ops.rs:35` |
| **Severidad** | MEDIO |
| **Categoría** | Storage / Overflow |

**Descripción:** `let new_size = (vstore.size * 2).max(total_needed + 4096);` — overflow si `size > 2^63`.

**Recomendación:** Usar `vstore.size.saturating_mul(2)`.

---

### MED-12: Integer overflow en alineación

| Campo | Detalle |
|---|---|
| **Archivo** | `src/storage/ops.rs:57` |
| **Severidad** | MEDIO |
| **Categoría** | Storage / Overflow |

**Descripción:** `vstore.write_cursor = (total_needed + 63) & !63;` — overflow si `total_needed > u64::MAX - 63`.

---

### MED-13: Falta fsync de directorio después de rename WAL

| Campo | Detalle |
|---|---|
| **Archivo** | `src/wal.rs:376, 410`; `src/storage/archive.rs:126` |
| **Severidad** | MEDIO |
| **Categoría** | WAL / Durabilidad |

**Descripción:** En filesystems ext4/XFS, el rename no es durable hasta fsync del directorio padre.

---

### MED-14: WAL corrupto truncado sin backup

| Campo | Detalle |
|---|---|
| **Archivo** | `src/wal.rs:231-239` |
| **Severidad** | MEDIO |
| **Categoría** | WAL / Recuperación |

**Descripción:** Los bytes corruptos se eliminan permanentemente sin cuarentena. Data que podría ser recuperable se pierde.

---

### MED-15: batch_append no es realmente batched

| Campo | Detalle |
|---|---|
| **Archivo** | `src/wal_sharded.rs:81-90` |
| **Severidad** | MEDIO |
| **Categoría** | WAL / Rendimiento |

**Descripción:** Lock/unlock por cada registro en vez de agrupar por shard primero.

---

### MED-16: parse_segment_timestamp fallback a mtime

| Campo | Detalle |
|---|---|
| **Archivo** | `src/wal_archiver.rs:283-297` |
| **Severidad** | MEDIO |
| **Categoría** | WAL / PITR |

**Descripción:** Si el filename no tiene timestamp parseable, se usa mtime. PITR replay no determinista si archivos son copiados/restaurados.

---

### MED-17: Condiciones relacionales solo soportan strings

| Campo | Detalle |
|---|---|
| **Archivo** | `src/parser/mod.rs:147-151` |
| **Severidad** | MEDIO |
| **Categoría** | Parser / Limitación |

**Descripción:** `edad > 18` es parse error. Todo valor debe estar entre comillas: `edad > "18"`. Comparaciones son lexicográficas, no numéricas: `"9" > "10"` sería true.

---

### MED-18: lang="es" hardcoded en layout SSR

| Campo | Detalle |
|---|---|
| **Archivo** | `web/src/app/layout.tsx:85` |
| **Severidad** | MEDIO |
| **Categoría** | Frontend / i18n |

**Descripción:** Mismatch de hidratación SSR cuando el language provider cambia el lang via JS.

---

### MED-19: Badge "v0.1 · MVP" obsoleto en hero

| Campo | Detalle |
|---|---|
| **Archivo** | `web/src/components/vanta/hero.tsx:62` |
| **Severidad** | MEDIO |
| **Categoría** | Frontend / Contenido |

**Descripción:** El workspace está en 0.4.0 pero el hero muestra v0.1.

---

### MED-20: Dependencia muerta next-auth

| Campo | Detalle |
|---|---|
| **Archivo** | `web/package.json:57` |
| **Severidad** | MEDIO |
| **Categoría** | Frontend / Dependencies |

**Descripción:** `next-auth@^4.24.11` nunca es importado en ningún archivo fuente.

---

### MED-21: Skip-link hardcoded en español

| Campo | Detalle |
|---|---|
| **Archivo** | `web/src/app/layout.tsx:91` |
| **Severidad** | MEDIO |
| **Categoría** | Frontend / i18n |

**Descripción:** "Saltar al contenido" no usa la función `t()` de i18n.

---

### MED-22: collection_delete hace deletes O(n) individuales

| Campo | Detalle |
|---|---|
| **Archivo** | `vantadb-mcp/src/lib.rs:1439-1445` |
| **Severidad** | MEDIO |
| **Categoría** | MCP / Rendimiento |

**Descripción:** Loop de deletes individuales sin batch. La transacción no envuelve los deletes.

---

### MED-23: active_requests counter leak en MCP

| Campo | Detalle |
|---|---|
| **Archivo** | `vantadb-mcp/src/lib.rs:508, 568-569` |
| **Severidad** | MEDIO |
| **Categoría** | MCP / Concurrency |

**Descripción:** Si `dispatch_request` panica antes de la línea 569, `active_requests.fetch_sub` nunca se ejecuta. No hay guard `Drop`.

---

### MED-24: Deserialización postcard de datos no confiables

| Campo | Detalle |
|---|---|
| **Archivo** | `fuzz/fuzz_targets/fuzz_node_deserialize.rs:17-20` |
| **Severidad** | MEDIO |
| **Categoría** | Seguridad / Input |

**Descripción:** `postcard::from_bytes` en datos de storage persistente. Mitigado con fuzz targets pero sin validación de bounds explícita en production paths.

---

### MED-25: noImplicitAny: false en web

| Campo | Detalle |
|---|---|
| **Archivo** | `web/tsconfig.json:13` |
| **Severidad** | MEDIO |
| **Categoría** | Frontend / TypeScript |

**Descripción:** Permite que `any` implícito prolifere, derrotando la seguridad de tipos.

---

### MED-26: Toast "quickstart.py copiado" hardcoded en español

| Campo | Detalle |
|---|---|
| **Archivo** | `web/src/components/vanta/code-terminal.tsx:117` |
| **Severidad** | MEDIO |
| **Categoría** | Frontend / i18n |

**Descripción:** No respeta el idioma seleccionado por el usuario.

---

## 4. ERRORES BAJO

> **Impacto:** Código muerto, nombres confusos, limpieza técnica.

---

### BAJO-01: Raíz hardcodea edition/rust-version

| Campo | Detalle |
|---|---|
| **Archivo** | `Cargo.toml:4-5` |
| **Categoría** | Configuración |

El crate raíz podría heredar vía `.workspace = true` como hacen los demás miembros.

---

### BAJO-02: version: "3.9" obsoleto en docker-compose

| Campo | Detalle |
|---|---|
| **Archivo** | `docker-compose*.yml:1` |
| **Categoría** | Infraestructura |

Compose v2 ignora la clave `version`. Emita warning.

---

### BAJO-03: Sin targets macOS/WASM en rust-toolchain.toml

| Campo | Detalle |
|---|---|
| **Archivo** | `rust-toolchain.toml:6` |
| **Categoría** | DX |

Solo Windows-MSVC y Linux-gnu. macOS y WASM requieren `rustup target add` manual.

---

### BAJO-04: RUSTSEC-2023-0089 ignorado hasta 2027

| Campo | Detalle |
|---|---|
| **Archivo** | `deny.toml:9` |
| **Categoría** | Dependencias |

`atomic-polyfill` unmaintained, transitive via postcard 1.1. Migración a postcard 2.0 pendiente.

---

### BAJO-05: tokio duplicado en vantadb-server

| Campo | Detalle |
|---|---|
| **Archivo** | `vantadb-server/Cargo.toml:12, 31` |
| **Categoría** | Configuración |

Presente en `[dependencies]` y `[dev-dependencies]` con features diferentes. Redundante.

---

### BAJO-06: MetricCache sobre-ingeniería para constante 2.0

| Campo | Detalle |
|---|---|
| **Archivo** | `src/index/distance.rs:47-63` |
| **Categoría** | Código |

`OnceLock<MetricCache>` solo para mantener `factor: 2.0`. Usar literal directamente.

---

### BAJO-07: purge_expired asigna registros completos innecesariamente

| Campo | Detalle |
|---|---|
| **Archivo** | `src/sdk/api.rs:446-506` |
| **Categoría** | Rendimiento |

Construye `VantaMemoryRecord` completo (incluyendo clonar vector y payload) solo para obtener `node_id`.

---

### BAJO-08: Cosine->Euclidean fallback silencioso para zero queries

| Campo | Detalle |
|---|---|
| **Archivo** | `src/index/search.rs:504-514` |
| **Categoría** | Index / Lógica |

Un query de norma cero con Cosine cambia a Euclidean silenciosamente. Rango de scores cambia.

---

### BAJO-09: Campo last_offset muerto y engañoso

| Campo | Detalle |
|---|---|
| **Archivo** | `src/wal_shipping.rs:48, 118-125` |
| **Categoría** | Código muerto |

Se setea como total acumulado (no por-segmento como implica el nombre) y nunca se lee.

---

### BAJO-10: Bloque dead code en hero

| Campo | Detalle |
|---|---|
| **Archivo** | `web/src/components/vanta/hero.tsx:166-195` |
| **Categoría** | Frontend / Código muerto |

`{false && (...)}` es inalcanzable. `setHeroVariant` existe pero el UI está permanentemente deshabilitado.

---

### BAJO-11: Tokenizer Python duplicado

| Campo | Detalle |
|---|---|
| **Archivo** | `web/src/components/vanta/code-playground.tsx:9-53`, `code-terminal.tsx:17-99` |
| **Categoría** | Frontend / Duplicación |

Misma lógica de tokenizer copiada en dos archivos.

---

### BAJO-12: Nombre genérico de paquete web

| Campo | Detalle |
|---|---|
| **Archivo** | `web/package.json:2` |
| **Categoría** | Frontend / Configuración |

`nextjs_tailwind_shadcn_ts` — nombre de boilerplate nunca actualizado.

---

### BAJO-13: ERROR_CODES definido y descartado

| Campo | Detalle |
|---|---|
| **Archivo** | `vantadb-ts/src/errors.ts:9-16, 18` |
| **Categoría** | SDK TypeScript / Código |

`void ERROR_CODES` descarta el valor. El tipo `ErrorCode` deriva de él pero no se enforcea en runtime.

---

### BAJO-14: Error interno filtrado a clientes MCP

| Campo | Detalle |
|---|---|
| **Archivo** | `vantadb-mcp/src/lib.rs:292-295` |
| **Categoría** | MCP / Seguridad |

`format!("{{\"error\":\"Serialization failed: {}\"}}", e)` expone detalles internos.

---

### BAJO-15: Flag --mcp sin registro en help

| Campo | Detalle |
|---|---|
| **Archivo** | `vantadb-server/src/main.rs:27` |
| **Categoría** | Server / DX |

`std::env::args().any(|a| a == "--mcp")` — no aparece en `--help`.

---

## 5. ASPECTOS POSITIVOS

| Área | Evaluación |
|------|------------|
| **Comparación de tokens (timing attack)** | CORRECTO — Usa `subtle::ConstantTimeEq` |
| **Rate limiting por IP** | CORRECTO — LRU con 1000 IPs, 5 intentos/60s |
| **Parsing anti-inyección** | CORRECTO — Parser `nom` estructurado, identifiers restringidos |
| **AES-256-GCM nonce** | CORRECTO — Nonce aleatorio por operación, sin reuso |
| **Supply chain security** | CORRECTO — `deny.toml` estricto: solo crates.io, sin wildcards |
| **Fuzz testing** | CORRECTO — 4 fuzz targets (parser, deserialize, WAL, archive) |
| **Unsafe inventory** | CORRECTO — Documentado exhaustivamente en `UNSAFE_INVENTORY.md` |
| **Miri testing** | CORRECTO — Tests de memoria presentes |
| **Default bind address** | CORRECTO — `127.0.0.1` (localhost) |
| **Sin secretos hardcodeados** | CORRECTO — Ningún secreto real en el código |
| **Diseño modular** | CORRECTO — Separación limpia entre módulos |
| **Build profiles** | CORRECTO — Dev, CI, y Release profiles bien configurados |
| **Clippy configuration** | CORRECTO — Workspace lints configuradas para todos los miembros |
| **Error handling en tests** | CORRECTO — Tests de seguridad dedicados en `tests/security/` |
| **RBAC** | FUNCIONAL — Sistema de roles básico con permisos granulares |
| **TLS** | CORRECTO — rustls con TLS 1.2 + 1.3 |

---

## 6. DISTRIBUCIÓN POR CATEGORÍA

```
Código Lógica/Algoritmos    ████████████████████  18 hallazgos
Seguridad                   █████████████████     14 hallazgos
Build/Infraestructura       ██████████████         12 hallazgos
Configuración               ████████               8 hallazgos
WAL/Durabilidad             ██████████             10 hallazgos
Frontend/SDKs/Integraciones ██████████████         12 hallazgos
Código Muerto/Limpieza      ████████               8 hallazgos
```

---

## 7. DEPENDENCIAS — VERSIONES DUPLICADAS

| Crate | Versiones | Severidad |
|-------|-----------|-----------|
| **hashbrown** | 0.17.1, 0.16.1, 0.15.5, 0.14.5 | MEDIO — 4 versiones |
| **rand** | 0.9.5, 0.8.7, 0.10.2 | MEDIO — 3 versiones |
| **rand_core** | 0.9.5, 0.6.4, 0.10.1 | MEDIO — 3 versiones |
| **getrandom** | 0.4.3, 0.3.4, 0.2.17 | MEDIO — 3 versiones |
| **windows-sys** | 0.61.2, 0.60.2, 0.59.0, 0.52.0 | BAJO — platform, unavoidable |
| **reqwest** | 0.13.4, 0.12.28 | MEDIO — 2 major versions |
| **thiserror** | 2.0.19, 1.0.69 | BAJO — transitive |
| **lru** | 0.16.4, 0.12.5 | BAJO — root pins 0.16 |

---

## 8. PLAN DE ACCIÓN RECOMENDADO

### Fase 1 — Bloqueantes (Esta Semana)

| # | ID | Acción |
|---|---|---|
| 1 | CRIT-06 | Establecer `flush_threshold` por defecto cuando `SyncMode::Periodic` |
| 2 | CRIT-07/08 | Arreglar Dockerfile (eliminar dirs falsos, actualizar Rust a 1.94.1+) |
| 3 | CRIT-10 | Definir features `prometheus` y `rayon` o eliminar código muerto |
| 4 | ALTO-01 | Reordenar `parse_literal_field_value` (float antes de parse_i64) |

### Fase 2 — Seguridad y Datos (Próxima Semana)

| # | ID | Acción |
|---|---|---|
| 5 | CRIT-01 | Corregir `ShardedWal::recover` con matemática round-robin |
| 6 | CRIT-04 | Propagar errores de tombstone con `tracing::error!` |
| 7 | ALTO-03 | Reemplazar SHA-256 por Argon2id/PBKDF2 para claves cortas |
| 8 | ALTO-04 | Validar `X-Forwarded-For` solo desde proxy confiable |
| 9 | ALTO-05 | Agregar `DefaultBodyLimit::max(1_000_000)` al router |
| 10 | MED-13 | Agregar `fsync` de directorio después de rename WAL |

### Fase 3 — Lógica de Producto

| # | ID | Acción |
|---|---|---|
| 11 | ALTO-02 | Invalidar/reconstruir índice IVF en `CPIndex::add()` |
| 12 | CRIT-09 | Mover providers a `workspace.exclude` o agregar `[workspace]` propio |
| 13 | MED-08 | Corregir paginación para usar conteo post-filtro |
| 14 | MED-05 | Retornar error en vez de descartar vectores norma-cero |
| 15 | MED-07 | Definir ordenamiento total para NaN en `NodeSim` |

### Fase 4 — Calidad y Mantenibilidad

| # | ID | Acción |
|---|---|---|
| 16 | CRIT-02 | Validar longitud del slice origen en `compact_layout` |
| 17 | ALTO-08 | Mantener lock del shard durante sync+open+replace |
| 18 | ALTO-09 | Persistir conteo de shards en metadatos |
| 19 | ALTO-14/15 | Bump integraciones a 0.4.0, pin `vantadb-py>=0.4.0,<0.5.0` |
| 20 | ALTO-12/13 | Habilitar `ignoreBuildErrors: false`, arreglar `isMemoryRecord()` |

---

## 9. NOTAS ADICIONALES

### Sin dependencias circulares
El grafo de dependencias internas es un DAG limpio:
- `vantadb` (raíz, sin deps internas)
- `vantadb-python` -> `vantadb`
- `vantadb-mcp` -> `vantadb`
- `vantadb-server` -> `vantadb` + `vantadb-mcp`
- `vantadb-wasm` -> `vantadb`
- `providers/{openai,ollama,litellm}` -> `vantadb`

### Sin secretos hardcodeados
Todos los secretos se cargan de variables de entorno. Ningún API key, password o token real encontrado en el código fuente. Los strings tipo-secret en `tests/security/security_audit.rs` son fixtures de test.

### Tests de seguridad dedicados
Existe un archivo `tests/security/security_audit.rs` con tests para:
- SQL injection (IQL parser)
- Timing attacks en auth
- Key length validation
- Null byte injection
- Input size limits

### Fuzz testing
4 targets en `fuzz/fuzz_targets/`:
- `fuzz_parser.rs` — Parser IQL
- `fuzz_node_deserialize.rs` — Deserialización de nodos
- `fuzz_wal.rs` — Headers y records WAL
- `fuzz_archive.rs` — Operaciones de archivo

---

> **Nota:** Esta auditoría fue realizada de forma estática (lectura de código) sin ejecución en runtime. Se recomienda validar los hallazgos CRÍTICOS con pruebas de integración antes de aplicar fixes en producción.

---

*Generado el 2025-07-27 por auditoría automatizada multi-agente sobre la rama `develop` (commit `63b0101d`).*
