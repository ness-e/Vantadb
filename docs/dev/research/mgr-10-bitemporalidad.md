# MGR-10 — Bitemporalidad (dim 5): valid-time vs transaction-time, tradeoffs y plan de migración (Cierre MGR)

- **Fecha:** 2026-09-28 · **Tipo:** investigación + diseño (cero implementación productiva; doc-only)
- **Contrato (plan Task 23):** "research-doc cerrado con modelo valid-time vs transaction-time, tradeoffs (append-only vs invalidación + storage del historial) y plan de migración/backfill determinista, listo para SCH-01"
- **Plan:** `docs/dev/plans/2026-09-26-master-roadmap.md` Task 23 (F3) · **Fuente:** Backlog:839 (P49, dim 5/AM4) + Notion dim 5 (Propuesta Anexo A) + decisión owner 2026-09-24 (migración única MGR-10/12/13)
- **Destraba:** SCH-01 (ADR-046) → SCH-02 (schema v2), SCH-03 (AS OF), SCH-06 (tests) · **Next:** SCH-01
- **Alcance:** 0.8.0 (corte F3). Índices temporales, MVCC y bitemporal per-fact completo = **v1.0** (§3). El ADR formal se consolida en SCH-01 — acá va el insumo.

> **Cierre MGR (Backlog:934 = research-doc + preguntas owner + plan de implementación):** §5 (preguntas owner) + §6 (plan) cierran el ciclo; §1–§4 son el insumo técnico para SCH-01.

## §0. Gap y estado del arte en el código (verificado contra HEAD 2026-09-28)

```bash
rg -n "valid_at|valid_from|bitemporal|as_of" src/   # → 0 hits (exit 1)
rg -n "point-in-time" src/                          # → solo snapshots/métricas de storage, no dominio temporal
```

Los hits de "point-in-time" son MVCC/snapshot de storage (`src/storage/engine/txn.rs:227`, `src/backends/rocksdb_backend.rs:350`, `src/metrics/core/mod.rs:622`) y comentarios afines — semántica transaccional/de proceso, **no** ventanas de validez de contenido. **Gap de dominio confirmado: VantaDB no tiene valid-time.**

Infra existente que el modelo debe reutilizar sin rediseño (4 piezas):

1. **Transaction-time parcial — historial por versión** (`src/sdk/version_history.rs`): snapshot post-commit en cada `put`, partition `BackendPartition::Versions`, key `ns_len(u32 LE)‖ns‖key_len(u32 LE)‖key‖version(u64 BE)` (:1-8); lectura `get_version` (:219-231) y `versions` (:233-250), API pública VS-CORE-07 (`src/sdk/api/memory.rs:409-432`). Retención FIFO con `version_history_limit` (default **32** por key, `src/config.rs:319`; evicción :252+). Durabilidad **best-effort post-commit** (:10-14): un crash entre el commit WAL y el snapshot deja *gap de versión, nunca corrupción*; crash-exactitud (`WalRecord::VersionSnapshot`) es deuda diferida P27.
2. **Invalidación explícita key→key** (ADR-028): `superseded_by`/`superseded_at_ms` (`src/sdk/types/record.rs:129-137`), API `supersede()` (`src/sdk/api/memory.rs:535-597`, con lock + guard de idempotencia; actualiza el registro vivo: `updated_at_ms=now`, `version+1`; WAL no atómico — 2 appends, aceptado en ADR-028), filtro de lectura `exclude_superseded` (list `record.rs:156-159`; search `src/sdk/serialization/vector_types.rs:120-123`; assembly `src/sdk/search/page.rs:317-319`). **No es ventana de validez**: es una marca de reemplazo.
3. **Schema header versionado** (`src/schema.rs:11` `CURRENT_SCHEMA_VERSION=1`; `TooOld`/`TooNew` :84-98; `.vanta.schema` :165-179) + maquinaria de migración ya existente: `vanta migrate plan|check|run <dir> [--format vfile|index|wal|schema|all] [--dry-run]` (`src/cli.rs:406-431`, `src/migration.rs` `FormatKind` :11-56, `plan_all` :119, `check_integrity` :359). Hoy la pata `schema` solo reescribe el header (`src/cli_handlers/migrate.rs:247-275`) — **no existe backfill de datos**.
4. **Export/import JSONL** con `EXPORT_SCHEMA_VERSION=1` (`src/sdk/serialization/mod.rs:35`), `export_line_from_record` (:498-514), `record_from_export_line` **rechaza** `schema_version != 1` (:522-531); CLI `vanta export --namespace <ns> --out <path>` / `vanta import --in <path>` (`src/cli.rs:106-128`).

**Formatos de persistencia que un campo nuevo atraviesa** (mapa de migración, §4.2):

| # | Formato | Referencia | Qué pasa con campos nuevos |
|---|---------|-----------|----------------------------|
| 1 | Node fields en KV/WAL (payload postcard) | `src/sdk/serialization/mod.rs:20-30` (`FIELD_*`), strip :340-345, read :352-357, write :446-464; `src/wal.rs:18,23,94-123` (`WAL_POSTCARD_VERSION` en header) | Aditivo: campo `__vanta_*` nuevo en el map del nodo; WAL forward-compat por versión de header |
| 2 | Mirror `SnapshotRecord` (postcard, **oculto**) | `src/sdk/version_history.rs:76-90`; `From` :112-148 | **Riesgo**: postcard posicional no tiene defaults por campo; hoy el mirror **omite `superseded_*`** y el `From` los resetea a `None` (:144-145) → el historial de versiones no reconstruye la invalidación. Todo campo nuevo exige mirror + roundtrip propios |
| 3 | Export JSONL | `MemoryExportLine` (`record.rs:229-261`), `serialization/mod.rs:498-514` | `#[serde(default)]` + bump de `EXPORT_SCHEMA_VERSION`; v1 debe seguir importable (SCH-02) |
| 4 | Header `.vanta.schema` | `src/schema.rs:11` | Bump 1→2 **al final** de la migración (marcador de "migración completa", §4.4) |

Borde fuera del alcance de records: `Edge` tiene 5 campos sin properties ni ventana de validez (`src/node/edge.rs:9`) → SCH-09 (Backlog:946), decisión en §4.5.

## §1. Modelo de dos ejes

### §1.1 Definiciones (literatura y estándar)

- **Valid time** = el período durante el cual el dato refleja correctamente la realidad, *según el usuario* (Snodgrass 1999; SQL:2011: "the time period during which a row is regarded as correctly reflecting reality by the user of the database" — Kulkarni & Michels 2012 §2.1).
- **Transaction time** = el período durante el cual el dato estuvo registrado/comprometido en la base ("committed to or recorded in the database"), mantenido **por el sistema**, no modificable por el usuario (SQL:2011 system-versioned tables).
- Fowler llama a los ejes *actual* (valid) y *record* (transaction): "on Mar 25th, we thought Sally's salary on Feb 25th was $6500" — la historia *actual* puede modificarse retroactivamente; la historia *record* es **append-only** y registra *cómo cambió nuestro conocimiento*.
- SQL:2011 fija precedentes directamente reutilizables: períodos **cerrados-abiertos** `[start, end)`; `FOR SYSTEM_TIME AS OF <ts>` (default `CURRENT_TIMESTAMP`); tablas bitemporales = application-time (valid, la escribe el usuario) + system-versioned (transaction, la escribe el sistema).
- **Agentes/LLM:** Zep/Graphiti implementan la misma semántica a nivel grafo: "each fact ... has a validity window: when it became true, and when (if ever) it was superseded"; "old facts are invalidated — not deleted". Es el precedente de producto más cercano a VantaDB (memoria de agentes), y ya está referenciado por el plan/Backlog.

**Terminología recomendada para VantaDB:** conservar `valid`/`transaction` (literatura + ADR-046 lo esperan) y nombrar los campos con el sufijo `_ms` del proyecto.

### §1.2 Los dos ejes, mapeados a VantaDB hoy

| Eje | Hoy | Cobertura | Límites |
|-----|-----|-----------|---------|
| **Transaction time** | Parcial: `version` + snapshots por versión (`Versions`) + `updated_at_ms`/`superseded_at_ms` | Por **key** (`get_version`/`versions`); sin queries cross-key | Retención ≤32/key; best-effort post-commit (gap de crash); `SnapshotRecord` pierde `superseded_*` |
| **Valid time** | **No existe** (0 hits) | — | — |

Nota de precisión: `updated_at_ms` es el tiempo de registro del *estado del registro* (bump en cada put/supersede), no una ventana de verdad; `superseded_at_ms` es el **evento de invalidación** (eje record). Ninguno expresa "cuándo era verdad el contenido" — eso es lo que agrega `valid_at`/`invalid_at`.

### §1.3 Modelo propuesto para 0.8.0 (recomendación para SCH-01/ADR-046)

**Campos nuevos en `MemoryRecord`** (nombres del Backlog:939; ADR decide finales):

- `valid_at_ms: u64` — inicio de validez (obligatorio tras backfill; default de lectura = `created_at_ms` si el campo falta — compat v1).
- `invalid_at_ms: Option<u64>` — fin de validez; `None` = abierto (∞).

**Semántica:** intervalo **cerrado-abierto** `[valid_at_ms, invalid_at_ms)` (SQL:2011). Predicado "vale en T":

```
valid_at_ms <= T && (invalid_at_ms.is_none() || invalid_at_ms > T)
```

**Compatibilidad de wire:** ambos `#[serde(default)]`; `invalid_at_ms: None` es el default natural; `valid_at_ms` ausente se **normaliza** en materialización (`record_from_node` → `unwrap_or(FIELD_CREATED_AT_MS)`) y en import v1 (explícito `valid_at = created_at`). Alternativa considerada: `Option<u64>` con normalización en query — más honesta en el wire crudo, pero obliga a normalizar en cada consumidor; se prefieren `u64` + normalización centralizada (SCH-01 ratifica).

**Relación con `supersede` (dos ejes, no redundancia):**

- `invalid_at_ms` = fin de **verdad** (valid time; puede ser retroactivo: "la política venció el mes pasado, lo supimos hoy").
- `superseded_at_ms` = cuándo la DB **registró** la invalidación (transaction time).
- Invariante por defecto (propuesto): `supersede()` escribe `invalid_at_ms = Some(now)` junto a `superseded_at_ms = Some(now)`. Divergen solo si un cliente fija `invalid_at` retroactivo — API dedicada diferida a v1.0 (§3); el **backfill** los alinea (`invalid_at = superseded_at`, Backlog:939).

**Time-travel:**

- **Por key (0.8.0, reutiliza lo existente):** `get_version(ns,key,v)` / `versions(ns,key)` ya responden "cómo era el registro en la escritura v". Un `as_of` de transaction-time se resuelve con la última versión retenida con `updated_at_ms <= R`. Limitaciones a documentar: acotado por retención (32/key) y por gap de crash; **sin** `as_of` transaction en search/list (no hay índice de versiones cross-key — v1.0).
- **Por validez (0.8.0, SCH-03):** `AS OF T` en IQL + parámetros equivalentes en search/list (filtro de ventana). Precedentes en el repo: filtros `MemoryFilter` con `Gt/Lt/Gte/Lte` + `Value::DateTime` (`record.rs:11-55`); integración en assembly **sin tocar índices** (patrón `exclude_superseded`, `page.rs:317-319`) y hasheada en el fingerprint del cursor (`page.rs:157`). Extensión de `exclude_superseded`: excluir también cuando `invalid_at_ms <= now` (SCH-03 decide flag aditivo vs extensión de semántica).
- **Default sin cambios:** las queries actuales siguen devolviendo todo (comportamiento actual intacto); el filtrado temporal es **opt-in**, como `exclude_superseded` hoy. Esto difiere del default de SQL:2011 (`AS OF CURRENT_TIMESTAMP`) pero evita breaking silencioso; SCH-01 lo ratifica explícitamente.

### §1.4 Invariantes propuestos (handoff SCH-01/02/03/06)

1. `valid_at_ms <= invalid_at_ms` cuando ambos están set (validado en write path + test).
2. **Backfill función pura:** `valid_at := created_at_ms`; `invalid_at := superseded_at_ms` si `superseded_by.is_some()`, si no `None`. Sin reloj, sin aleatoriedad → misma DB v1 ⇒ mismo resultado byte-idéntico (§4.1).
3. `supersede()` ⇒ `invalid_at_ms == superseded_at_ms` por defecto.
4. Lectura v1-compat: campo `valid_at_ms` ausente ⇒ `created_at_ms`; `superseded_*` ausente ⇒ `None`.
5. Filtros temporales no cambian el orden del ranking ni el cursor salvo que el request los incluya (hash de cursor, precedente `page.rs:157`).

## §2. Tradeoffs

### §2.1 Append-only vs invalidación (mecanismo de historia)

| Criterio | **Invalidación** (recomendado 0.8.0) | **Append-only / system-versioned** |
|---|---|---|
| Precedente | Zep/Graphiti (invalidar ≠ borrar); ADR-028 ya implementado (`supersede` + `exclude_superseded`); snapshots por versión | SQL:2011 system-versioned; Snodgrass transaction-time tables |
| Cambio requerido | Campos + backfill + queries de filtro (aditivo) | **Rediseño del engine**: store append-only + retención/GC propios; los índices derivados (HNSW/text/scalar) hoy apuntan al nodo vivo |
| Storage | Marca de 2 campos + snapshots completos por versión (cap 32) | Cada versión = fila inmutable completa; duplica storage |
| Atomicidad | `supersede` = 2 appends WAL (aceptado ADR-028; 2PC es ACID Phase 0) | Insert único de versión |
| Consultas | AS OF valid + invalid por filtro; transaction-time solo por key (snapshots) | AS OF ambos ejes sin límite de retención |
| Riesgo | Historia transaction acotada (cap/gap) — documentable | Rompe modelo live-record + hot path de escritura; **stop condition del plan** ("si exige rediseño del engine → acotar a records + FIND") |

**Recomendación:** invalidación ahora (mecanismo ADR-028 extendido con ventana de validez) + snapshots como transaction-time parcial; system-time real per-fact queda v1.0.

### §2.2 Storage del historial

| Opción | Pros | Cons | Veredicto 0.8.0 |
|---|---|---|---|
| (a) Full-record snapshots por versión (actual) | Ya existe; simple; roundtrip probado; cap configurable | Costo O(versiones × tamaño registro); FIFO por key; best-effort (gap de crash) | ✅ mantener |
| (b) Deltas por versión | Menos storage | Encode/decode + reconstrucción ≠ trivial; postcard manual; riesgo de corrupción silenciosa | ❌ |
| (c) WAL-exact `WalRecord::VersionSnapshot` (deuda P27) | Sin gaps; base audit-grade | Cambio de formato WAL + replay; costo de write en hot path | v1.0 (evaluar con VER-01) |
| (d) Solo marker (sin payload histórico) | Trivial | Pierde "qué era verdad en T" — incumple el DoD (evidence-before-belief, Backlog:839) | ❌ |

### §2.3 Descartadas (rabbit holes del plan)

- **MVCC / índices temporales** en 0.8.0 → NO (filtros en assembly, "no index change").
- **Segundo store de eventos** (CQRS) → NO (segundo camino de escritura + doble fuente de verdad; One-Version).
- **Period type SQL-like / temporal grammar completa** → NO (minimal: campos + parámetros + clauses gateadas).

## §3. 0.8.0 vs v1.0 (anti-scope-creep)

| Capacidad | 0.8.0 (corte F3) | v1.0 / diferido (dueño) |
|---|---|---|
| Campos `valid_at_ms`/`invalid_at_ms` en record + backfill determinista | ✅ (SCH-02) | — |
| `AS OF T` valid-time en IQL + search/list | ✅ (SCH-03) | — |
| Filtros por ventana + `exclude_superseded` extendido | ✅ (SCH-03) | — |
| Time-travel por key (transaction) | ✅ reutiliza `get_version`/`versions` | — |
| `as_of` transaction-time cross-key en search | ❌ (sin índice) | v1.0 (índice temporal) |
| `recorded_from/to` per-fact (bitemporal completo Snodgrass) | ❌ | v1.0 |
| Crash-exactitud del historial (WAL-exact) | ❌ (gap documentado) | v1.0 / P27 (VER-01) |
| Retención per-namespace / políticas | ❌ (cap global actual) | v1.0 |
| Edges con ventana de validez | ⚠️ decisión SCH-01 (§4.5: entrar a la migración única v2 o FIND) | — |
| Agregados/joins temporales, MVCC | ❌ | v1.0 (fuera de stop conditions) |

## §4. Plan de migración / backfill determinista

### §4.1 Reglas de backfill (deterministas por construcción)

```
valid_at_ms   := created_at_ms
invalid_at_ms := superseded_at_ms
```

- Ambas fuentes **ya están persistidas** en cada registro → función pura del estado v1; **prohibido** leer reloj (`now_ms()`), aleatoriedad u orden de iteración.
- **Idempotente:** re-ejecutar recalcula los mismos valores (los campos migrados no alimentan el cálculo).
- **Order-independent:** transformación por registro; los chunks no afectan el output — misma DB v1 ⇒ resultado byte-idéntico (gate SCH-02/SCH-06).

### §4.2 Qué toca cada formato

1. **Node fields (KV/WAL):** agregar `FIELD_VALID_AT_MS` / `FIELD_INVALID_AT_MS` (`src/sdk/serialization/mod.rs:20-30`), incluirlos en el set `__vanta_*` a stripes del metadata de usuario (:340-345) y en read/write (:352-357, :446-464). El backfill escribe los nodos existentes (batched; `engine.insert`/write batch).
2. **`SnapshotRecord` (formato oculto postcard):** append de campos al final del mirror + `From` bidireccional (incluye por fin `superseded_*` — fix del gap :144-145). Decodificación con **fallback V2-first → V1**: postcard `from_bytes` (docs.rs) *ignora bytes sobrantes*, así que V1-decoding-V2-bytes "triunfaría" en silencio si se intenta primero; V2-decoding-V1-bytes falla por falta de campos → el fallback ordenado es determinista. **Verificar en SCH-02** con test de bytes V1 reales (si postcard cambiara esa semántica → prefijo de versión en el value). Re-encode siempre V2.
3. **Export JSONL:** `MemoryExportLine` + campos `#[serde(default)]`; `EXPORT_SCHEMA_VERSION → 2`; `record_from_export_line` acepta `{1, 2}` (v1 se normaliza: `valid_at=created_at`, `invalid_at=superseded_at`); export emite siempre v2. Tests: roundtrip v2 + import v1 existente (contrato SCH-02).
4. **Header `.vanta.schema`:** `CURRENT_SCHEMA_VERSION → 2`, `MIN_COMPAT_VERSION` se mantiene en 1 (binario v2 lee DB v1 vía normalización; binario v1 sobre DB v2 → `TooNew`, correcto porque no sabe decodificar snapshots V2). El bump es el **marcador de migración completa** y va **último**.

### §4.3 Secuencia con comandos (expand → backfill → bump)

```bash
# 0. Baseline (regla dura -p; nunca nextest sin -p desde la raíz)
cargo nextest run --profile audit -p vantadb --build-jobs 2

# 1. Plan (dry-run informativo; hoy muestra vfile/index/wal/schema → se extiende con "records: N")
vanta migrate plan <dir>

# 2. Integridad pre-migración
vanta migrate check <dir>

# 3. Copia de respaldo (para doble corrida determinista; o `vanta snapshot create <name>`)
Copy-Item -Recurse <dir> <dir>.v1-copy

# 4. Backfill en seco (reporta N + muestra)
vanta migrate run <dir> --format records --dry-run

# 5. Backfill real (idempotente; crash-safe: re-run completa)
vanta migrate run <dir> --format records

# 6. Cierre: resto de formatos + bump de header (siempre DESPUÉS del backfill)
vanta migrate run <dir> --format all

# 7. Verificación de determinismo: dos copias de la DB v1 → migrar ambas → comparar
vanta migrate run <copyA> --format all && vanta migrate run <copyB> --format all
#    → hash/cmp byte a byte de los artefactos (fixture versionado)

# 8. Roundtrip export/import (v2 y v1)
vanta export --namespace <ns> --out export-v2.jsonl
vanta import --in export-v2.jsonl
vanta import --in tests/fixtures/export-v1.jsonl   # v1 sigue importable
```

Notas: `records` es un `FormatKind`/paso nuevo a definir en SCH-02 (hoy `FormatKind` = vfile|index|wal|schema, `src/migration.rs:11-56`); si SCH-02 lo integra como sub-paso de `schema`, mantener semántica "header al final". Los tests de determinismo viven en SCH-06 (doble corrida byte-idéntica + crash mid-migración con failpoints/SIGKILL existentes).

### §4.4 Crash-safety e invariantes de migración

- Orden **expand → backfill → bump**: un crash mid-backfill deja header v1 + nodos parcialmente backfilled. Ambos binarios siguen leyendo: v2 normaliza (`unwrap_or created_at`); v1 ignora campos extra del map y decodifica snapshots V2 de forma degradada (trailing ignorado) sin corrupción. Re-run completa el backfill.
- La parte más frágil es la **re-encode de snapshots** (formato oculto): failpoint dedicado + SIGKILL (infra existente) + doble corrida en SCH-06.
- Sin `now()` en el backfill ⇒ la determinismo no depende de reloj inyectado; si un paso futuro lo necesitara, aplica reloj inyectado (pre-mortem SCH-06).

### §4.5 Borde edges (SCH-09) — fuera del core, decisión en SCH-01

`Edge` (`src/node/edge.rs:9`) no tiene `properties` ni ventana. Backlog:946 propone incluirlo en la **misma migración v2** (`valid_at_ms`/`invalid_at_ms` + backfill `valid_at=created_at_ms`). Este research-doc **no diseña el esquema de edges** (stop condition: si exige segundo breaking o rediseño de queries de grafo → FIND con dueño). Recomendación: SCH-01 decide con el insumo de SCH-09; default = incluirlo en la migración única si el costo es acotado, FIND si no.

## §5. Preguntas owner (Cierre MGR)

1. **Default de queries:** ¿mantener el comportamiento actual (sin filtro de validez por defecto, filtros opt-in como `exclude_superseded`) o imitar SQL:2011 (`AS OF now` por defecto)? *Recomendado: mantener — cero breaking; `AS OF` es explícito.*
2. **`invalid_at` retroactivo:** ¿API 0.8.0 permite fijar `invalid_at` en el pasado (efecto bitemporal real) o 0.8.0 solo lo deriva de `supersede`? *Recomendado: campo existe con semántica completa; setter dedicado v1.0; 0.8.0 lo escribe `supersede`.*
3. **Retención del historial:** ¿cap global 32/key (config actual) o política per-namespace en 0.8.0? *Recomendado: cap global + documentar límite; per-namespace v1.0.*
4. **Snapshots históricos:** ¿migración in-place obligatoria (garantiza `superseded_*` + campos nuevos en todo el historial retenido) o aceptar pérdida de esos campos en versiones viejas con FIND? *Recomendado: in-place — el costo está acotado por el cap y cierra el contrato de migración.*
5. **Crash-exactitud (P27):** ¿promover `WalRecord::VersionSnapshot` en 0.8.0 o diferir a v1.0 con gap documentado? *Recomendado: v1.0 (evaluar con VER-01).*
6. **Edges (SCH-09):** ¿entran a la migración única v2 o FIND? *Recomendado: decidir en SCH-01 con insumo SCH-09 (§4.5).*

## §6. Plan de implementación (consume SCH-01 → SCH-02/03/06)

1. **SCH-01 (ADR-046):** modelo §1.3 + semántica exacta (nombres finales, cerrado-abierto, defaults, compat v1) + decisión de preguntas owner + tabla de reconciliación con MGR-12/13 + plan de migración de §4 verbatim.
2. **SCH-02 (schema v2 + backfill):** campos + reglas §4.1 + migración de los 4 formatos §4.2 + `FormatKind`/paso `records` + tests deterministas (doble corrida) + import v1.
3. **SCH-03 (queries):** `AS OF` (IQL + params) + filtros de ventana + extensión de `exclude_superseded` + cursor fingerprint + `IQL_VERSION` bump.
4. **SCH-06 (tests):** migración doble byte-idéntica, crash mid-migration (failpoint/SIGKILL), time-travel con fechas de referencia, bordes (TTL+valid, supersede+invalid, versión evictada), roundtrip v1↔v2.
5. **Docs (mismo PR, Regla 3):** `docs/api/EMBEDDED_SDK.md` + `docs/api/IQL.md` + `docs/api/VERSIONING.md` (política de breaking 0.8.0).

## §7. Fuentes externas verificadas (GATE CITAS, 2026-09-28)

| Fuente | URL | Verificación |
|---|---|---|
| Snodgrass, *Developing Time-Oriented Database Applications in SQL*, Morgan Kaufmann, 1999 (ISBN 1-55860-436-7) | <https://www2.cs.arizona.edu/~rts/tdbbook.pdf> | Resuelve (HTTP 200, `application/pdf`; enlace canónico desde <https://www2.cs.arizona.edu/~rts/publications.html> ✅) |
| Kulkarni & Michels (IBM), "Temporal features in SQL:2011", SIGMOD Record 41(3), sept. 2012 — periods, application-time, system-versioned, bitemporal, `FOR SYSTEM_TIME AS OF` | <https://sigmodrecord.org/publications/sigmodRecord/1209/pdfs/07.industry.kulkarni.pdf> | ✅ contenido completo vía Jina Reader (`r.jina.ai`; fetch directo bloqueado por Cloudflare) |
| Fowler, "Bitemporal History", 07-abr-2021 — ejes actual/record, record-history append-only, storage (rangos bitemporales vs event sourcing) | <https://martinfowler.com/articles/bitemporal-history.html> | ✅ fetched |
| Rasmussen et al., "Zep: A Temporal Knowledge Graph Architecture for Agent Memory", arXiv:2501.13956 (2025) | <https://arxiv.org/abs/2501.13956> | ✅ fetched |
| Graphiti (Zep) — "Facts have validity windows... old facts are invalidated, not deleted"; "bi-temporal tracking with automatic fact invalidation" | <https://github.com/getzep/graphiti> | ✅ fetched |
| XTDB — DB inmutable con historial por defecto sobre capacidades bitemporales de SQL:2011 (precedente de producto viable) | <https://docs.xtdb.com/> | ✅ fetched |
| postcard `from_bytes` — el remanente de bytes no consumidos no se devuelve (base del fallback V2→V1 de §4.2) | <https://docs.rs/postcard/latest/postcard/fn.from_bytes.html> | ✅ fetched (postcard 1.1.3) |

## Deuda y notas

- **Deuda nueva:** ninguna (research + docs-only; saldo neto Regla 6 = 0).
- **NOTICED BUT NOT TOUCHING:** (a) `SnapshotRecord` omite `superseded_*` (pre-existente; lo aborda SCH-02 §4.2-2); (b) deuda P27 (snapshots WAL-exact) citada, no creada; (c) edge-bitemporal (SCH-09) fuera del diseño (stop condition) con recomendación en §4.5; (d) `src/` no tocado.
- **Skills (SDP):** deprecation-and-migration (pinned storage/schema — expand→backfill→bump de §4), writing-plans (§6), writing-guidelines, source-driven-development (§7 con URLs), coordinated-web-search (cascada keyless + fallback Jina), context-engineering; base: campaign-executor/progreso (auto).
