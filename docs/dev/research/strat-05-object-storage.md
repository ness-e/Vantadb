---
title: "STRAT-05 - Ruta object storage (S3/blob): viabilidad, costo y tradeoffs"
kind: research
status: active
description: "Research de la ruta object storage: snapshot-level vs segment-level vs backend nativo, con costos verificados (S3/R2) y decisión documentada en ADR-0056; sin implementación"
tags: [vantadb, research, storage, s3, object-storage]
---

# STRAT-05 - Ruta object storage (S3/blob): viabilidad, costo y tradeoffs

- **Fecha:** 2026-10-06 · **Tipo:** research/spec (STRAT-05, Task 65, plan 0.9.0) · **Cero implementación** (este run no genera código ni símbolos)
- **Contrato (plan Task 65 L1867):** research + spec de la ruta object storage (viabilidad, costo, tradeoffs: snapshot-level vs segment-level vs backend nativo) con decisión documentada (ADR si toca el modelo de storage); sin implementación; fuentes citadas (Regla 11).
- **Origen:** fila `STRAT-05` del Backlog (removida al cierre de esta tarea — registro en [`../avance/activo/operaciones.md`](../avance/activo/operaciones.md)) + [BIZ-14](../Backlog-negocio.md) (backup offsite S3/red, trigger Pro/Cloud) + veredicto 2026 ([`feature-verdicts-2026.md`](../archive/research-old/feature-verdicts-2026.md) §Backup/Restore y §"fuera del roadmap") + [ADR-0004](../architecture/adr/ADR-0004-storage-backend.md)/[ADR-0020](../architecture/adr/ADR-0020-storage-backend-default.md) + [STORAGE-TIERS](../architecture/STORAGE-TIERS.md).
- **Alcance:** decidir viabilidad/costo/tradeoffs de la ruta object storage **antes** de que el trigger Pro/Cloud la fuerce sin diseño. **No** implementa; **no** re-litiga el veredicto 2026 ("no S3 todavía; S3 es Fase 5"; "❌ S3 backup nativo cloud only") ni BIZ-14 — los cita y declara la frontera. La decisión vive en [ADR-0056](../architecture/adr/ADR-0056-object-storage-path.md).

## Contenido

[§0 Resumen ejecutivo](#0-resumen-ejecutivo) · [§1 Estado real](#1-estado-real-del-storage-vantadb) · [§2 Opciones](#2-las-tres-opciones) · [§3 Costos](#3-modelo-de-costos-supuestos-marcados) · [§4 Decisión](#4-decisión-recomendada-y-condiciones) · [§5 Fuentes](#5-fuentes-verificadas-2026-10-06)

## §0. Resumen ejecutivo

| Pregunta | Respuesta |
|---|---|
| ¿La DB puede vivir hoy en object storage (backend nativo)? | **No hoy.** Requiere un motor nuevo (sin mmap, sin fsync local, latencia 50-100ms/request): los precedentes que lo logran construyeron **formatos/motores nuevos** (Lance, SlateDB), se limitaron a **read-only** (DuckDB) o delegaron el locking a la app con riesgo de corrupción (sqlite-s3vfs). VantaDB depende de mmap (`vstore_L*.vanta`) y de la semántica local del WAL/LSM. |
| ¿Cuál es la ruta viable ahora? | **Snapshot-level** (subir/bajar snapshots `.vantadb`/directorios): encaja con las semánticas de object storage (objeto entero atómico), cero cambios al motor, y es exactamente el alcance de **BIZ-14**. |
| ¿Y la continuidad (RPO bajo)? | **Segment-level** (ship de WAL/segmentos estilo Litestream) al **trigger Pro/Cloud** (PRO-02/03): RPO de segundos/minutos sin tocar el camino caliente; requiere un subsistema de replicación + manifiesto + restore (spec aparte cuando dispare). |
| ¿Cuánto cuesta? | Modelo §3: backup diario de una DB de 10 GB con retención ≈ **$2.5-3.5/mo en S3** o **≈$1.65/mo en R2** (egress $0); WAL shipping ≈ **$1-1.5/mo**. Los costos de request son despreciables frente al storage; el costo real del nativo es de **ingeniería**, no de factura. |
| ¿Qué cambia en código con este run? | **Nada.** Decisión + spec + ADR; la implementación de BIZ-14 queda como fila/plan futura con la decisión ya tomada. |
| ¿Reabre el veredicto 2026? | **No.** Lo confirma y lo refina: se mantiene "no S3 nativo ahora"; se decide **qué sí** (snapshot-level) y **bajo qué condiciones** se revisa (trigger Pro/Cloud → segment-level; serverless/edge como requisito de producto → re-evaluar nativo). |

## §1. Estado real del storage VantaDB

**Hoy 100% local** (verificado contra el worktree, HEAD `b57bf445`):

| Componente | Evidencia | Rol |
|---|---|---|
| Backend KV | `src/backend.rs:105-132` — `BackendKind { RocksDb, Fjall (default), InMemory }`; construcción vía `BackendRegistry` `src/storage/engine/init.rs:275-283`; trait `pub(crate)` (`src/backend.rs:227`) | Sin variante object storage; contrato interno |
| Vector store | [STORAGE-TIERS](../architecture/STORAGE-TIERS.md) — segmentos `vstore_L0..L3.vanta` **mmap**, un archivo por nivel LSM | **mmap ⇒ no puede vivir en object storage** (no seek/read parcial; `object_store` lo declara por diseño: "stateless APIs instead of cursor based interfaces such as Read or Seek") |
| WAL | ADR-0002 (CRC32C + auto-healing), ADR-0038 (fsync batching opt-in, *proposed*) | Camino caliente local; el WAL es el candidato natural a *shipping* (segment-level) |
| Backup **local ya existe** | CLI `Backup`/`Restore` (`src/cli.rs:146-168`); `Snapshot Create/List` hard-links (`src/cli.rs:409-419`); engine `create_snapshot`/`snapshot_restore` (`src/storage/engine/mod.rs:647,:697,:795`); SDK `create_snapshot`/`restore_from` (`src/sdk/builder.rs:281,:309`); lógico `export_all`/`import_file` (`src/sdk/serialization/impl_export.rs:315,:531`), `bulk_import_file` (`src/sdk/api/memory.rs:2515`) | **Falta solo el tramo "offsite"** (subir/bajar a red) = BIZ-14 |
| Gap físico documentado | [`res02-backup-restore.md`](archive/res02-backup-restore.md) (archivado 2026-08-25) | Historia: `wal_archiver.rs` **ya no existe** (re-verificado); `snapshot_restore` ya existe — el gap remanente es el transporte offsite |

**Restricciones estructurales para cualquier opción remota:** (a) no hay escritura parcial de objetos (S3 reemplaza el objeto entero; multipart permite re-subir una *parte*, pero no bytes in-place); (b) no hay mmap/seek remoto; (c) la latencia de object storage es 50-100ms/request (vs µs local) — cualquier cosa en el camino caliente cambia el perfil del motor; (d) single-writer: el locking entre clientes queda a cargo de la aplicación (o de primitivas condicionales).

## §2. Las tres opciones

### 2.1 Snapshot-level (backup/DR) — **VIABLE YA**

**Qué es:** exportar un snapshot consistente (directorio `.vantadb` con `Backup`/`Snapshot`, o export lógico `export_all`) y subirlo como **objeto(s) enteros** a S3/blob; restore = bajar y restaurar (`Restore`/`import_file`).

**Precedentes:** veredicto 2026 ("Sí, pero no S3. Solo snapshot local primero" → este run extiende el "solo local" al transporte remoto con diseño); RES-02 (backup físico); cualquier backup de archivo entero sobre S3 (semántica natural del objeto).

**Tradeoffs:**
- ✅ Cero cambios al motor ni al wire; RPO = cadencia de backup (horas/días); restore simple (bajar objeto + `Restore`).
- ✅ Encaja con las semánticas de S3 (PUT atómico de objeto entero; multipart para trozos grandes).
- ✅ Implementable como adaptador fino con el crate `object_store` (Apache Arrow): S3/GCS/Azure/R2/local con la misma API, async, producción (crates.io, InfluxDB IOx).
- ⚠️ Costo de storage proporcional a copias × tamaño (ver §3); mitigable con retención/lifecycle.
- ⚠️ RPO no es "segundos": para continuidad real hace falta segment-level.

### 2.2 Segment-level (WAL/segment shipping) — **VIABLE AL TRIGGER Pro/Cloud**

**Qué es:** replicación asíncrona del WAL (y de los segmentos `vstore`) a object storage, estilo Litestream: el proceso de replicación lee páginas/segmentos nuevos, los empaqueta (TXID + checksums) y los sube; restore = replay. El motor sigue 100% local (el camino caliente no cambia).

**Precedentes:** Litestream (SQLite WAL → LTX → S3; RPO ~segundos; proceso separado); SQLite CBS (bloques fijos 4MB + manifest — versión "block-level" con escritura); Chroma (log + índices en object storage, compactors async, caché SSD local).

**Tradeoffs:**
- ✅ RPO de segundos/minutos sin tocar el write path local (asíncrono).
- ✅ Costo bajo (§3: ~$1-1.5/mo de escenario) — PUTs pequeños son baratos.
- ⚠️ Subsistema nuevo: manifiesto de estado, coordinación con checkpoints/compaction, tooling de restore, tests de crash/replay. Es **spec aparte** (no cabe en este run; es la implementación que BIZ-14/PRO-02/03 consumirían).
- ⚠️ Consistencia entre WAL y segmentos mmap (orden de subida y point-in-time del restore) — el diseño tiene que fijar el invariante (p.ej. snapshot base + delta WAL).

### 2.3 Backend nativo (`BackendKind::ObjectStore`) — **DIFERIDO (no es un variant, es un motor nuevo)**

**Qué es:** que las operaciones KV del motor vivan directamente sobre object storage (la DB "en el bucket"), con disco local solo como caché.

**Precedentes y qué costó cada uno:**
- **SlateDB** (Rust, Apache-2.0): LSM **construido desde cero** para object storage; "object storage request latencies are an order of magnitude higher (~50-100ms per request)"; es un **proyecto/motor nuevo** (near 1.0, producción en Dropbox et al), no un backend intercambiable de un motor local.
- **Lance/LanceDB**: **formato nuevo** optimizado para object storage (fragmentos/manifests/random access) — mismo patrón: el formato se rediseñó para el medio.
- **DuckDB**: soporte **read-only** (`ATTACH ... (READ_ONLY)`); no hay connect r/w sobre S3.
- **sqlite-s3vfs / CBS**: block-layer VFS — S3 no soporta reemplazo parcial ("to change even 1 byte, it must be re-uploaded in full"); sin locking la DB "will probably become corrupt"; CBS delega el single-writer a la aplicación.
- **SlateDB FAQ:** "one PUT per write gets expensive" → por eso batching/SST/LSM; es decir, hasta el motor nuevo existe **porque** la semántica KV directa sobre S3 es cara/incómoda.

**Tradeoffs (contra VantaDB hoy):**
- ❌ mmap imposible ⇒ rediseño del acceso a `vstore`; ❌ fsync/durabilidad locales ⇒ primitivas condicionales/fencing nuevas; ❌ latencia 50-100ms ⇒ perfil de motor distinto (SlateDB lo asume, VantaDB no); ❌ single-writer/multi-reader + compaction remota = multi-trimestre.
- ⚠️ Solo tendría sentido como **requisito de producto nuevo** (serverless/edge sin disco) y aun así sería "otro motor", no una variante de `BackendKind`.

## §3. Modelo de costos (supuestos marcados)

> **Precios verificados con fecha** (Regla 11); **supuestos de escenario marcados** como tales. List prices, región US estándar.

**Precios (fetch 2026-10-06):**

| Ítem | S3 Standard (us-east-1, Price List API 2026-09-28) | R2 (página 2026-10-01) |
|---|---|---|
| Storage | $0.023/GB-mo (primeros 50TB) | $0.015/GB-mo |
| PUT/COPY/POST/LIST | $0.005/1.000 | Class A $4.50/1M |
| GET | $0.004/10.000 | Class B $0.36/1M |
| Egress | $0.09/GB (10TB, tras 100GB free) | **$0** |
| Infrequent/IA | $0.0125/GB-mo + $0.01/GB retrieval | IA $0.01/GB-mo + $0.01/GB retrieval |

**Escenario A1 — snapshot-level (supuestos: DB 10 GB; backup diario full; retención 7 diarios + 4 semanales ≈ 110 GB-mo; 1 restore/mes):**

| Componente | S3 | R2 |
|---|---|---|
| Storage 110 GB-mo | ≈ $2.53/mo | ≈ $1.65/mo |
| PUTs (11-30/mo) | ≈ $0 | ≈ $0 |
| Restore (10 GB egress) | ≈ $0.90 | $0 |
| **Total** | **≈ $2.5-3.5/mo** | **≈ $1.65/mo** |

**Escenario A2 — segment-level (supuestos: 1 segmento/min ≈ 43.2k PUTs/mo; 50 GB-mo incrementales; restore con miles de GETs):**

| Componente | S3 | R2 |
|---|---|---|
| PUTs 43.2k | ≈ $0.22/mo | ≈ $0.19/mo |
| Storage 50 GB-mo | ≈ $1.15/mo | ≈ $0.75/mo |
| GETs restore | céntimos | céntimos |
| **Total** | **≈ $1.4/mo** | **≈ $1/mo** |

**Escenario A3 — backend nativo** (supuesto: ~1M writes/día batched en commits, ≈30k PUTs/mo): el driver no es la factura (PUTs ≈ $0.15-0.30/mo) sino la **ingeniería** (motor/formato nuevo, multi-trimestre) y el perfil de latencia (~50-100ms/request). Referencia: EFS $0.30/GB-mo + $0.03/GB reads + $0.06/GB writes (SlateDB FAQ) → incluso "block storage de red" es 20x el storage de S3; object storage es para DR/cold/replicación, no para el camino caliente.

**Supuestos marcados (no mediciones):** tamaño de DB, cadencia, retención, volumen incremental. Compresión/dedup no modeladas (jugarían a favor del snapshot-level con export comprimido). Los precios son list prices con fecha; AWS puede variar por región/descuentos.

## §4. Decisión recomendada y condiciones

**Decisión (→ [ADR-0056](../architecture/adr/ADR-0056-object-storage-path.md), status `proposed` — ratificación del owner):**

1. **Ahora (la decisión):** la ruta object storage de VantaDB es **snapshot-level** — BIZ-14 la implementará como transporte offsite de snapshots (CLI/SDK) sobre `object_store`, **sin tocar el motor**, cuando su trigger Pro/Cloud dispare (la implementación NO arranca con este run). Local-first intacto (ADR-0004/0020 vigentes).
2. **Al trigger Pro/Cloud (PRO-02/03):** diseñar **segment-level** (WAL/segment shipping con manifiesto + restore) como capa de continuidad/DR — spec dedicada cuando dispare; este research fija el porqué y el cómo (precedentes Litestream/CBS/Chroma).
3. **Backend nativo:** **diferido**, con condiciones de revisión explícitas: (a) requisito de producto serverless/edge sin disco local; (b) y aun así se trataría como **motor nuevo** (precedentes Lance/SlateDB), no como variante de `BackendKind`.

**Frontera declarada (pre-mortem #2 — no re-litigio):**
- **Veredicto 2026** ([`feature-verdicts-2026.md:128,135-137,387`](../archive/research-old/feature-verdicts-2026.md)): "Sí, pero no S3. Solo snapshot local primero" / "❌ S3 backup nativo (cloud only, Fase 5+)". → Se **mantiene** "no S3 nativo ahora"; este run decide el *cómo* del backup offsite (snapshot-level) y deja el nativo condicionado.
- **BIZ-14** ([`Backlog-negocio.md:103`](../Backlog-negocio.md)): "Exportar instantáneas `.vantadb` a almacenamiento de red; trigger Pro/Cloud; Dep PRO-02/03". → No se duplica: este research le entrega la decisión de diseño; su implementación sigue siendo la fila de negocio.
- **Frontera:** *backup offsite* (habilitable por snapshot-level) ≠ *storage path* (la DB viviendo en object storage = diferido).

**Triggers de revisión del nativo:** (1) requisito serverless/edge sin disco; (2) costos de block storage de red (EFS-like) dominando el TCO de un cliente; (3) madurez de un crate LSM-on-object-store reutilizable (SlateDB 1.0+) que permita *evaluar* (no adoptar en caliente). Hasta entonces, la decisión no se re-litiga sin evidencia nueva.

## §5. Fuentes verificadas (2026-10-06)

| # | Fuente (fetch 2026-10-06) | Qué se extrajo |
|---|---------------------------|----------------|
| 1 | https://slatedb.io/index.md | SlateDB: LSM sobre object storage; 50-100ms/request; single-writer/multi-reader; producción; near 1.0 |
| 2 | https://slatedb.io/docs/get-started/faq/ | "one PUT per write gets expensive"; WAL a object storage; EFS $0.30/GB-mo |
| 3 | https://litestream.io/how-it-works/ | WAL→LTX (TXID+checksums); replicación, no storage remoto |
| 4 | https://sqlite.org/cloudsqlite/doc/trunk/www/index.wiki | CBS: bloques fijos + manifest; single-writer por app; Azure/GCS |
| 5 | https://github.com/dpedu/sqlite-s3vfs | "S3 does not support the partial replace of an object"; sin locking → corrupción |
| 6 | https://duckdb.org/docs/current/guides/network_cloud_storage/duckdb_over_https_or_s3 | DuckDB: conexión read-only vía HTTPS/S3 |
| 7 | https://github.com/duckdb/duckdb/discussions/10466 | `ATTACH ... (READ_ONLY)`; sin connect r/w |
| 8 | https://lance.org/format/ | Lance: formato optimizado para object storage; fragmentos/manifests |
| 9 | https://docs.trychroma.com/reference/architecture/distributed | Chroma: log + índices en object storage; catálogo SQL; SSD caché |
| 10 | https://www.trychroma.com/engineering/serverless | "35-100ms"; power-law; compactors async |
| 11 | https://kuzudb.github.io/docs/extensions/s3/ | Kuzu: read/write/glob de archivos vía httpfs (no DB en S3) |
| 12 | https://docs.rs/object_store/latest/object_store/index.html | object_store: uniforme S3/GCS/Azure; stateless (no Read/Seek) |
| 13 | https://developers.cloudflare.com/r2/pricing/ | R2: $0.015/GB-mo; egress $0 |
| 14 | https://www.devzero.io/blog/aws-s3-pricing (Price List API 2026-09-28) | S3: $0.023/GB-mo; PUT $0.005/1k; GET $0.004/10k |
| 15 | https://egresscost.com/aws/s3-egress-pricing/ | Egress $0.09/GB; S3→CloudFront gratis; cross-region $0.02/GB |
| 16 | https://docs.aws.amazon.com/AmazonS3/latest/userguide/mpuoverview.html | Multipart: partes independientes; no replace parcial |

**Evidencia interna:** `src/backend.rs:105-132`, `src/backend.rs:227`, `src/storage/engine/init.rs:275-283`, `src/storage/engine/mod.rs:647,:697,:795`, `src/sdk/builder.rs:281,:309`, `src/sdk/serialization/impl_export.rs:315,:531`, `src/sdk/api/memory.rs:2515`, `src/cli.rs:146-168,409-419`, [STORAGE-TIERS](../architecture/STORAGE-TIERS.md), [ADR-0004](../architecture/adr/ADR-0004-storage-backend.md), [ADR-0020](../architecture/adr/ADR-0020-storage-backend-default.md), [`res02-backup-restore.md`](archive/res02-backup-restore.md) (archivado), [`feature-verdicts-2026.md`](../archive/research-old/feature-verdicts-2026.md).
