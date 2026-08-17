# Memory Budget (Compute/Storage Separation) — Reglas

> **Scope:** presupuesto de memoria del motor: qué vive en RAM vs disco, límites de RSS, back-pressure antes de OOM. Aplica a `src/index/` (HNSW/flat), `src/storage/` (vstore, backends, LSM), `src/memory_governor.rs`, `src/cache_warmer.rs`, y a cualquier estructura nueva que retenga datos del usuario en RAM.
> **No tocar aquí:** políticas de fsync/durabilidad (→ `durability.md`), locks/async (→ `concurrency-async.md`), algoritmos de indexación (→ `indexes.md`).
> **Status:** 🟡 En revisión
> **Fuentes:** FND-01 (`docs/Investigaciones/FND-01-memory-budget.md`, benchmark `benches/memory_budget.rs`)

## Reglas

### 1 — Conocer qué vive en RAM y qué en disco (inventario normativo)

- **Must:** cualquier estructura nueva que retenga datos del usuario debe declarar su residencia en el reporte de diseño: RAM (heap/mmap residente) o disco (backend KV / vstore / WAL).
- **Must not:** no asumir que "está en disco" porque el dato se persiste — la copia viva puede seguir en RAM.
- **Por qué:** el inventario FND-01 confirma que HNSW (`src/index/graph.rs:145`, `HnswNode` en DashMap) y FlatIndex (`src/index/flat.rs:63`) son **100% RAM residentes**; el vstore es mmap a disco (`src/storage/vfile_mmap.rs`), el KV backend es LSM en disco (`src/backends/`), y la WAL es disco (`src/wal.rs`). Confundir persistencia con residencia produce OOM por sorpresa.

### 2 — Medir el RSS real del proceso, no estimaciones parciales

- **Must:** toda decisión de back-pressure por memoria debe basarse en el **RSS real del proceso** — `crate::metrics::core::_get_rss_virt()` (Win32 `GetProcessMemoryInfo` / Mach `task_info` / `/proc/self/statm`, fallback sysinfo en `src/metrics/core/mod.rs:471`) — o en una estimación lógica explícitamente documentada como tal. El guard `check_memory_pressure` (FND-01-F1, aplicado) usa `rss > 0 ? rss : effective_bytes()`: la medición en vivo del host con fallback a la estimación previa cuando la medición no está disponible (Miri / plataforma sin soporte). `memory_breakdown_snapshot().process_rss_bytes` es la misma medición pero solo se refresca en `flush()` — no es fuente para decisiones en vivo.
- **Must not:** usar `MemoryStats.physical_rss` (suma de `mmap_resident_bytes()` de vstores + backend HNSW) como única señal de presión. El benchmark FND-01 mide 54 MiB de mmap-residente contra 354 MiB de RSS real a 20k nodos (subestimación ~6.5×) — el guard cree que hay 0.2% de presión cuando el proceso usa 1.1% de la RAM y crece lineal con el dataset.
- **Por qué:** antes de FND-01-F1, `check_memory_pressure` (`src/storage/engine/stats.rs:98`) usaba `effective_bytes()` = `physical_rss.unwrap_or(logical)`; con `mmap_hnsw=true` (default) el `physical_rss` solo cuenta páginas mmap residentes y **oculta** el heap (structs HNSW, neighbor lists, WAL buffers, backend cache, text index). El RSS real ya se medía en `flush()` (`record_memory_breakdown`) pero no alimentaba el check → el guard no podía prevenir OOM. Desde FND-01-F1 el guard mide el RSS real en vivo (con fallback), cerró el blind spot: a 20k nodos el `pressure_ratio` pasó de 0.002 (mmap) a 0.011 (RSS real).

### 3 — Back-pressure antes de que el SO mate el proceso

- **Must:** cuando el RSS del proceso supera el umbral configurado (`rss_threshold`, default 0.80 × `memory_limit`), las escrituras deben rechazarse con `VantaError::ResourceLimit` y un mensaje que incluya bytes usados, umbral y sugerencia de acción — antes de alcanzar el límite físico.
- **Must not:** permitir que una estructura residente (HNSW, cache, tabla co-access) crezca sin límite acotado por configuración o watermark.
- **Por qué:** con RSS ~17–20 KB/nodo (HNSW 1536d, slope del benchmark), 1M nodos ≈ 20 GB y ~1.6M nodos ≈ RAM total de la máquina de referencia (31.78 GiB). Sin back-pressure efectivo el SO mata el proceso (OOM killer / `alloc` falla) sin oportunidad de degradación gradual.

### 4 — Toda estructura residente nueva lleva un cap declarado

- **Must:** estructuras en RAM con crecimiento dependiente de datos deben declarar un tope duro en el mismo módulo, con política documentada al alcanzarlo (evictar / dejar de aprender / rechazar): patrón existente: volatile cache (`src/storage/engine/insert.rs:303-311`, cap = `total_memory/4/1536` nodos), co-access (`src/cache_warmer.rs:30`, `MAX_CO_ACCESS_PAIRS`), cardinality (`src/config.rs`, `MAX_CARDINALITY_PAIRS`).
- **Must not:** agregar un HashMap/DashMap/vec sin tope cuyo tamaño dependa del dataset del usuario.
- **Por qué:** los caps existentes son la única razón por la que hoy no hay OOM *de esas estructuras*; HNSW, al ser el índice principal, no tiene cap — de ahí la regla 3.

<!-- Referencias cruzadas: → ver durability.md (WAL/fsync), indexes.md (HNSW), concurrency-async.md (locks) -->