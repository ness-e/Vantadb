# F3X — diseño trait-split storage↔index (hoja neutral, patrón M3)

## Metadata
- **Plan file:** docs/plans/2026-09-13-cleanCA-fase3.md (Wave 1, Task 2 — diseño-primero; tras F3G)
- **Creado:** 2026-09-14
- **Estado:** ⏳ IN PROGRESS (diseño completo, pendiente aprobación humana Gate D — sin código)
- **Ruta:** vanta-arch (diseño + ADR-datos) → vanta-worker (slices X1–X3 tras aprobación)
- **Appetite:** 2–3d (diseño ~0.5d en este file; resto develop / worker tras Gate D)
- ** Alcance ESTA tarea:** SOLO DISEÑO + este task file. CERO cambios en `src/`, cero slices de código.
- **nextTask:** F3C (tras F3X-impl del worker)
- **SDP:** phase=PLAN + contractKeywords [traits, architecture, ADR] (ver §5). Sin campaign MCP (adaptador plan §10).

## 1. TAREA — hipótesis Q3 (hoja neutral, patrón M3) vs código actual
- **Hipotesis inicial humana Q3 (Gate P.3):** hoja neutral, patrón M3 (`search_profile.rs` + `fusion.rs` como precedente). El arch la valida y la devuelve para aprobar.
- **Veredicto Gate D de diseño: HIPÓTESIS VALIDADA CON AJUSTE MENOR (GO con ajuste).**
  - El patrón M3-hoja aplica **íntegro** a la mitad del ciclo (flag + consts compartidas → kernel/hoja sin deps).
  - La otra mitad (CPIndex/IndexBackend como tipos concretos) **no admite inversión M3 pura** (las 8 fns de M3 eran puras sobre tipos sdk y tenían dueño natural; CPIndex es estado con backend mmap + 918L core + search/serialize — moverlo a storage violaría ownership derived-vs-canonical de BOUNDARIES §2). Requiere **trait-split (DIP/ISP)**: engine depende del trait, index lo implementa. Es la opción 1 del plan de BOUNDARIES §3 ("Trait-split task (preferred)"), no una alternativa nueva.
  - **Ajuste menor (HALLAZGO-H2):** no hace falta crear hoja nueva para el flag — el kernel YA posee `NodeFlags::TOMBSTONE = 1<<3 = 0x8` (`src/node/flags.rs:29`), idéntico valor a `FLAG_TOMBSTONE = 0x8` (`src/storage/engine/mod.rs:55`). La "hoja neutral" para el flag = migrar usos a la const kernel existente (cero archivos nuevos para esa arista).
- **Alternativa considerada y rechazada:** inversión total (mover CPIndex a storage o engine a index) — rechazada: index es derived/rebuildable, storage es canonical/durable (BOUNDARIES §2); fusionarlos colapsa el boundary que el gate quiere hacer explícito. Fusión = ocultar el síntoma, criterio M3 la prohíbe.
- **Box/re-export cosmético:** NO vale (pre-mortem). `pub use crate::index::FreshHnswReport` en `engine/mod.rs:35` se mantiene solo como compat BND-04; los participantes del ciclo deben importar la ruta neutral/trait, no el re-export.

## 2. ARCHIVOS
- **Clave (solo lectura en esta tarea — verificación file:línea del §7):**
  - `src/storage/archive.rs:4` (`use crate::index::CPIndex;`)
  - `src/storage/engine/mod.rs:34` (`use crate::index::CPIndex;`) + `:35` (`pub use crate::index::FreshHnswReport;` compat BND-04) + `:55` (`pub(crate) const FLAG_TOMBSTONE`)
  - `src/storage/engine/init.rs:12` (`use crate::index::{CPIndex, IndexBackend};` — plan decía :17, deriva trivial ya firmada en F3G/G3)
  - `src/storage/engine/maintenance.rs:9` (`use crate::index::{CPIndex, IndexBackend};`) + `:17` (`use crate::storage::vfile::MmapMut;`)
  - `src/index/mod.rs:20` (`use crate::storage::vfile::File;`, usado en `:59` firma `vector_store: Option<&File>`)
  - `src/index/serialize/file.rs:2` (`use crate::storage::vfile::MmapMut;` bajo `#[cfg(not(feature = "memmap2"))]`, `:3-4` alterna a `memmap2::MmapMut`)
  - `src/index/graph/types.rs:5` (`use crate::storage::vfile::MmapMut;` mismo gate de feature, `:61-67` `pub enum IndexBackend` posee `mmap: Option<MmapMut>`)
  - `src/index/search/layer.rs:13` (`use crate::storage::engine::FLAG_TOMBSTONE;`, hot-path `search_layer`, 5 sitios de uso `:115,118,288,291,346`)
  - `src/index/flat.rs:18` (fn-local `use crate::storage::engine::FLAG_TOMBSTONE;` dentro de `flat_search`, prod — qualifier corregido en F3G/G3)
- **Relacionados (solo lectura):**
  - `docs/architecture/BOUNDARIES.md` BND-02/BND-03 + §3 (tabla del ciclo + plan opción 1/2) + BND-04 (re-exports solo compat)
  - `docs/tasks/C2M3.md` (precedente: hoja `src/search_profile.rs` std+serde-only + inversión fns→`sdk/search/fusion.rs`; criterio Box/re-export)
  - `docs/tasks/F3G.md` (base: commit `dd892c4c`, gate 4/4, firma A1 árbol `5a1dae99`, tabla D)
  - `src/search_profile.rs:1-16` (doc de hoja neutral: cero intra-crate deps), `src/index/graph/core.rs:15` (`pub struct CPIndex`), `src/node/flags.rs:29` (`TOMBSTONE = 1<<3`), `src/storage/vfile.rs:110` (`pub struct File`), `src/storage/vfile_mmap.rs:26,208` (origen `MmapMut`), `src/storage/engine/mod.rs:325` (`hnsw: ArcSwap<CPIndex>`)
- **Prohibidos:**
  - Todo cambio en `src/` en ESTA tarea (diseño-only; ni siquiera el worker toca hasta Gate D).
  - Resto del repo fuera de los listados; WIP ajeno (`.opencode` M, `desktop/src-tauri/Cargo.lock` M — no tocar, no incluir).

## 3. DEPENDENCIAS
- **Base:** F3G ✅ COMPLETE (commit `dd892c4c`, gate 4/4: re-medición + BND-03X + firma A1). Sin F3G no hay base de medición.
- **Habilita:** F3X-impl (worker, slices X1–X3 del §contrato-slices) **solo tras aprobación humana de este diseño** (Gate D question del §ADR-question).
- **nextTask:** F3C (evita colisión en `storage/engine/init.rs` si el trait-split lo toca; plan Wave 2).
- **Riesgo heredado:** firmas `pub` = semver major — verificado en DISCOVERY: `CPIndex`/`IndexBackend`/`FreshHnswReport` son `pub`, pero el diseño NO cambia sus firmas (solo añade trait sellado + mueve usos a kernel); `FLAG_TOMBSTONE` es `pub(crate)` (sellado, sin riesgo semver). Detalle en §8.

## 4. REFERENCIAS
- M3 (`docs/tasks/C2M3.md`): inversión fns→sdk + hoja `search_profile.rs` (tipos+consts, `pub use` BND-04 solo compat, participantes a ruta neutral).
- Rust API Guidelines C-SEALED: traits públicos sellados con `pub(crate)` (sealed pattern) para permitir evolución sin major.
- Guía Clean §4.1 ISP/DIP: segregar puertos por rol; depender de abstracciones; el dueño del estado implementa, el orquestador consume el trait.
- BOUNDARIES §3 plan opción 1 (trait-split preferido) / opción 2 (DEFER explícito con fila FIND — no es "no tocar").

## 5. SKILLS (SDP phase=PLAN + keywords traits/architecture/ADR, 10 cargadas vía tool `skill`)
- campaign-executor — state machine DISCOVERY→diseño→CIERRE + task file + RESULTADO (siempre).
- progreso — Trigger 2 (grep previo anti-duplicado) + Trigger 1 al cierre del diseño (diseño-only: registra file, no commit).
- systematic-debugging — si la re-medición contradice A1/Fase 2, root-cause antes de declarar HALLAZGO (no reintentar a ciegas).
- code-review-and-quality — auto-revisión 5 ejes de este diseño antes del cierre (contrato tipado + tabla + slices).
- doubt-driven-development — verificación adversarial del diseño (asumir que la hoja oculta una arista; buscarla).
- source-driven-development — versiones/flags verificados contra fuentes (feature `memmap2` gates en file.rs:1-4/types.rs:4-7; C-SEALED de Rust API Guidelines [cita NO VERIFICADA — sin red]).
- planning-and-task-breakdown — slices verticales X1–X3 + checkpoints por slice (especificar, no ejecutar).
- codebase-memory — code intel (omitido por adaptador sin campaign MCP; sustituido por `rg` + Read directos — justificado, no por pereza).
- api-and-interface-design — Contract First del trait/hoja (firmas + errores + dirección deps + Hyrum surface) — **core de este diseño**.
- documentation-and-adrs — ADR-datos solo con datos (Regla 5: la decisión la escribe el humano; aquí §ADR-datos + question).

## 6. HERRAMIENTAS+MCP
- bash solo lectura (este diseño): `rg -n "CPIndex|IndexBackend|FLAG_TOMBSTONE|vfile::(File|MmapMut)|FreshHnswReport"` (re-medición §7) + `rg -n "storage::vfile" src/` (alcance vfile) + `git log/status` (base `dd892c4c`, worktree limpio salvo WIP ajeno).
- `codegraph_explore` / `check_index_coverage`: no disponibles por adaptador (sin campaign MCP para estos IDs); sustituidos por Read directos tras glob (glob-antes-de-Read cumplido §2).
- `Write` del task file: este file (`docs/tasks/F3X.md`) — único artefacto de escritura de la tarea.
- `question` tool: UNA ronda al cierre (§ADR-question). Si la tool no está disponible en el runner, la ronda se emite como BLOQUEO estructurado en el RESULTADO con opciones + recomendado.

## 7. INVESTIGACION CODIGO — re-medición de las 8 aristas en el árbol actual (`dd892c4c`)
Base: F3G firmó A1 contra `5a1dae99`; este diseño re-mide contra `dd892c4c` (F3G encima).

| # | Arista (dirección) | Evidencia actualizada | vs A1/Fase 2 | Trait/hoja propuesto |
|---|---|---|---|---|
| A1 | storage→index `CPIndex` | `src/storage/archive.rs:4` `use crate::index::CPIndex;` (usos `:44,173,199,208,222,231` fns con `&CPIndex`) | idéntica | `IndexPort` (trait §contrato): `archive.rs` consume trait, no tipo concreto |
| A2 | storage→index `CPIndex`+re-export | `src/storage/engine/mod.rs:34` `use crate::index::CPIndex;` + `:35` `pub use crate::index::FreshHnswReport;` | idéntica (F3G ya notó `:34-35`) | trait en hoja; `:35` se conserva como compat BND-04 y NO cuenta como migración |
| A3 | storage→index `CPIndex,IndexBackend` | `src/storage/engine/init.rs:12` (plan decía :17 — deriva trivial firmada F3G/G3, mismo contenido) usos `:39,312,326,332,378` (construye `CPIndex::new/load_from_file/with_backend`) | deriva solo de línea | `IndexPort` + `IndexBackendKind` (constructor abstracto; engine deja de nombrar `IndexBackend` concreto) |
| A4 | storage→index `CPIndex,IndexBackend` | `src/storage/engine/maintenance.rs:9` usos `:156,195,198` (rebuild: `deserialize_from_bytes`, asigna `.backend = MMapFile`) | idéntica | `IndexPort::rebuild_from_bytes` + setter de backend tras el trait (engine no toca el campo `.backend` directamente) |
| B1 | index→storage `File` | `src/index/mod.rs:20` `use crate::storage::vfile::File;` (uso `:59` firma `vector_store: Option<&File>`) | idéntica | `VectorStoreRef` (trait/reference abstracto §contrato); `mod.rs:20` se elimina |
| B2 | index→storage `MmapMut` | `src/index/serialize/file.rs:2` (gate `#[cfg(not(feature = "memmap2"))]`, alterna `:3-4` `memmap2::MmapMut`) usos `:73,148` (asigna/lee `.backend` mmap) | idéntica | `MmapBackend` (trait sellado §contrato); el `use` feature-gated se mueve a la hoja, index nombra el trait |
| B3 | index→storage `MmapMut` | `src/index/graph/types.rs:5` (mismo gate) + `:61-67` `pub enum IndexBackend` posee `mmap: Option<MmapMut>` + `:88-113` fns que llaman `crate::storage::vfile::{get_resident_bytes, Mmap::map, File::open}` | idéntica | `IndexBackend` implementa `MmapBackend`; los 3 calls a `storage::vfile` se encapsulan tras el trait (único módulo que sigue viendo vfile) |
| B4a | index→storage `FLAG_TOMBSTONE` (hot) | `src/index/search/layer.rs:13` + 5 usos `:115,118,288,291,346` en `search_layer` | idéntica | **`NodeFlags::TOMBSTONE` (kernel existente, HALLAZGO-H2)** — cero hoja nueva |
| B4b | index→storage `FLAG_TOMBSTONE` (fn-local) | `src/index/flat.rs:18` dentro de `flat_search` + uso `:35` | idéntica | **`NodeFlags::TOMBSTONE`** igual que B4a |

**HALLAZGO-H1 (alcance vfile mayor que la tabla, informativo — no cambia el diseño):** `storage::vfile` aparece además en firmas FQ sin `use` top-level: `src/index/diskann.rs:400`, `src/index/ivf.rs:424`, `src/index/scann.rs:239`, `src/index/search/nearest.rs:26,52`, `src/index/search/mod.rs:21`, `src/index/search/layer.rs:27`, `src/index/flat.rs:98` (`vector_store: Option<&crate::storage::vfile::File>`), más `src/node/vector_data.rs:5` (`Mmap`), `src/lsm.rs:132,134,156` (`File`). Si `cargo modules` cuenta paths FQ como aristas, X2 debe migrar esas firmas a `VectorStoreRef` también (el slice X2 lo incluye como sub-paso mecánico con `rg` de verificación). No dispara Gate V (dirección y dueño idénticos; solo conteo).
**HALLAZGO-H2 (kernel ya posee el flag — simplifica el diseño):** `src/node/flags.rs:29` `pub const TOMBSTONE: u32 = 1<<3` == `src/storage/engine/mod.rs:55` `pub(crate) const FLAG_TOMBSTONE: u32 = 0x8`. B4a/B4b se resuelven por migración a kernel (2 imports + `rg` sustitución), sin archivo nuevo. `FLAG_TOMBSTONE` queda como alias `pub(crate)` deprecado interno o se elimina en X3 (decisión del worker con clippy; sin efecto API por ser `pub(crate)`).

## 8. INVESTIGACION PROBLEMA — qué owns qué
- **Tipos:** `CPIndex` (`pub struct`, `src/index/graph/core.rs:15`, campos `pub`: `nodes/backend/config/...`) y `IndexBackend` (`pub enum`, `src/index/graph/types.rs:61`, variantes `InMemory/MMapFile{path,mmap}`) los OWNEA index. `File` (`pub struct`, `src/storage/vfile.rs:110`) y `MmapMut` (vía `src/storage/vfile_mmap.rs:26,208`, re-export `vfile.rs:32-33`) los OWNEA storage. `FLAG_TOMBSTONE` (`pub(crate) const 0x8`, `engine/mod.rs:55`) lo declara engine pero DUPLICA `NodeFlags::TOMBSTONE` del kernel (H2).
- **Box/re-export:** el único `pub use` en el ciclo (`engine/mod.rs:35` → `FreshHnswReport`) es compat BND-04 y no lo toca ningún participante del ciclo como ruta de importación (verificado: nadie hace `use crate::storage::engine::FreshHnswReport` para el ciclo). Mantenerlo NO oculta aristas; usarlo como "migración" SÍ las ocultaría → prohibido por criterio M3.
- **Holder del estado:** `StorageEngine` posee `hnsw: ArcSwap<CPIndex>` (`engine/mod.rs:325`) y lo CONSTRUYE (`init.rs:39` `new`, `:312` `load_from_file`, `:326` `with_backend(new_mmap)`, `archive.rs:208` `fresh_index_like`). Engine = holder/orquestador; index nunca construye engine. Dirección del trait: engine consume `IndexPort`, index lo implementa; la construcción pasa por factory del trait (`open_index`/`rebuild`) para que engine no nombre `CPIndex`/`IndexBackend` concretos.
- **Visibilidad/semver:** `FLAG_TOMBSTONE` es `pub(crate)` → migración a kernel sin bump. `CPIndex`/`IndexBackend` son `pub` → el diseño NO cambia sus firmas ni los elimina (adición de trait sellado = minor/aditivo; si el worker detecta un `pub` que deba cambiar → ADR + semver major documentado, precedente ADR-041).

## 9. INVESTIGACION INTERNET — digest (sin red en este runner)
- Sin ambigüedad de patrón que exija web: el precedente M3 local (`search_profile.rs` hoja std+serde-only + inversión documentada en C2M3 §Evaluación) y el plan votado en BOUNDARIES §3 opción 1 cubren el diseño. Sealed-traits (C-SEALED, Rust API Guidelines) e ISP/DIP (§4.1) se citan desde conocimiento del plan; al no poder re-verificarse en vivo se marcan **[cita NO VERIFICADA — sin red]** con verificación pendiente para F3X-impl (`contract.deuda`).
- Deuda de verificación: F3X-impl debe confirmar C-SEALED (sealed-trait pattern) e ISP/DIP contra fuentes oficiales antes de codificar el trait; si la fuente contradice el sellado propuesto → HALLAZGO + Gate V.

## CONTRATO TIPADO — hoja neutral + traits (Contract First)

**Hoja:** NEW `src/index_port.rs` (nombre a aprobar en question; alternativas §ADR-question) — cero intra-crate deps salvo `std + serde + crate::node + crate::error` (mismo perfil que `search_profile.rs:1-16`). Dirección: `storage/engine/*` → `crate::index_port::*` ← `index/*` (ambos dependen hacia abajo; la hoja no importa ni storage ni index).

```rust
//! Neutral index-port leaf (F3X): breaks the `storage ↔ index` module cycle.
//! Consumers depend DOWNWARD on this leaf (std + serde + node + error only).
//! Backward compat (BND-04): old paths stay alive as `pub use` for one minor;
//! cycle participants import from here directly.

use serde::{Deserialize, Serialize};

/// Rol de backend persistente sin exponer `MmapMut` concreto (ISP por rol).
#[non_exhaustive] // extensible sin major
pub enum IndexBackendKind { InMemory, MMapFile { path: std::path::PathBuf } }

/// Vista de solo-lectura del vector-store que el search necesita (B1 + firmas FQ H1).
pub trait VectorStoreRef: Send + Sync {
    fn resident_bytes(&self) -> Option<u64>;
}

/// Rol mmap tras el que se encapsulan los 3 calls a `storage::vfile` de types.rs (B2/B3).
/// Sellado: solo implementable dentro del crate (`pub(crate)` supertrait/blanket).
pub trait MmapBackend: Send + Sync { /* sellado: ver nota C-SEALED */
    fn mmap_path(&self) -> Option<&std::path::Path>;
    fn mmap_resident_bytes(&self) -> Option<u64>;
    fn sync_to_mmap(&mut self) -> std::io::Result<()>;
}

/// Puerto que engine consume y el index implementa (A1–A4). Idempotente en
/// rebuild; atómico donde el backend lo soporte (documentado por impl).
/// Errores: SOLO `crate::error::Error` (`VantaError`) — sin `panic`/`Option`/strings ad-hoc.
pub trait IndexPort: MmapBackend + Send + Sync {
    type Snapshot;
    fn open_index(path: &std::path::Path, use_mmap: bool) -> Result<Self, crate::error::Error> where Self: Sized;
    fn rebuild_from_bytes(data: &[u8], mmap_path: Option<std::path::PathBuf>) -> Result<Self, crate::error::Error> where Self: Sized;
    fn fresh_like(&self, index_path: std::path::PathBuf) -> Result<Self, crate::error::Error> where Self: Sized;
}
```

- **Hyrum surface (lo que se garantiza vs lo que NO):** se garantiza firma+error (`VantaError`) y dirección (engine→trait←index); NO se garantiza orden de iteración de `list`/neighbors ni texto de mensajes (documentar `unordered` donde aplique); el layout binario en disco no cambia en este split (cero migración de datos).
- **Evolución:** aditivo (nuevo trait + enum `#[non_exhaustive]`) → minor, sin bump major; `FLAG_TOMBSTONE` (`pub(crate)`) → alias interno o eliminación en X3 sin efecto público.
- **Validación:** solo en boundaries (constructores `open_index/rebuild_from_bytes` validan paths/bytes); core interno confía en tipos (sin re-validación duplicada).
- **Naming:** `IndexPort` (puerto), `VectorStoreRef`/`MmapBackend` (roles ISP), `IndexBackendKind` (dato sin comportamiento); `snake_case` fns (`open_index`, `rebuild_from_bytes`, `fresh_like`, `mmap_resident_bytes`); feature-gates intactos (`memmap2`/`failpoints` se mueven con su código, sin importar sin feature).

## SECUENCIA DE SLICES PARA EL WORKER (F3X-impl — especificar, NO ejecutar) + PLAN ACYCLIC/EDGE-AUDIT
- **X1 — hoja + migración storage→index (A1–A4):** crear `src/index_port.rs` + `pub mod index_port` en `lib.rs`; `impl IndexPort/MmapBackend/VectorStoreRef for CPIndex` en index (nuevo `src/index/port_impl.rs`, `pub(crate)`); migrar `archive.rs:4`, `engine/mod.rs:34`, `init.rs:12`, `maintenance.rs:9` a `crate::index_port::IndexPort` (+ `fresh_index_like`→`fresh_like`, constructor/rebuild→trait fns, asignación `.backend`→trait setter). Verify por sub-paso: `cargo check -p vantadb` + `rg "use crate::index::(CPIndex|IndexBackend)" src/storage/` → 0.
- **X2 — migración index→storage (B1–B3 + H1):** firmas `vector_store: Option<&File>` (mod.rs:59, layer.rs:27, nearest, mod, diskann/ivf/scann/flat `_vector_store`) → `Option<&dyn VectorStoreRef>`; `file.rs:2` + `types.rs:5` → trait (`MmapMut` solo visible en `port_impl.rs`/hoja bajo el mismo `cfg`); `types.rs:88-113` encapsulados; B4a/B4b → `NodeFlags::TOMBSTONE` (con `use crate::node::NodeFlags;`). Verify: `rg "storage::(engine::FLAG_TOMBSTONE|vfile::(File|MmapMut))" src/index/` → 0 + `rg "storage::vfile" src/index/ src/node/ src/lsm.rs` revisado (solo quedan usos legítimos fuera de index o migrados).
- **X3 — suites + acyclic + edge-audit + cierre:** `cargo check -p vantadb --tests --all-targets` + `clippy -D warnings` + `fmt` + `nextest -p vantadb --lib storage index` verdes; `cargo modules dependencies --lib -p vantadb --acyclic` sin aristas storage↔index (artefacto accumulator BND-03X exceptuado); edge-audit `rg` de las 8 + H1 como evidencia primaria (acyclic es best-effort con timeout conocido en Windows); alias `FLAG_TOMBSTONE` eliminado o deprecado interno; vanta-review visa el diff antes del commit del lead. Sin cambio semántico (moves + trait, cero lógica nueva; hot path solo movido, no optimizado).

## ADR-DATOS (solo datos — Regla 5: la decisión la escribe el humano)
- **Contexto (datos):** ciclo bidireccional storage↔index verificado A1/Fase 2 y re-medido en este file (§7, 8/8 aristas presentes en `dd892c4c`); engine es holder de `ArcSwap<CPIndex>` y constructor; kernel ya posee el flag (`NodeFlags::TOMBSTONE == FLAG_TOMBSTONE == 0x8`); vfile tiene 25+ importadores (moverlo = grande); M3 precedent valida hoja+inversión pero CPIndex no admite inversión (estado con backend).
- **Opciones (datos):** A) hoja `index_port.rs` + `IndexPort/MmapBackend/VectorStoreRef` sellados (este diseño; aditivo, minor, ~3 archivos nuevos + migraciones mecánicas); B) mover `vfile` a kernel top-level (rompe B(i) de raíz pero toca 25+ importadores + `lsm/node/migration`; grande); C) DEFER explícito con fila FIND + revisit date (opción 2 de BOUNDARIES §3; costo 0, el gate de mudanza queda sin prerrequisito).
- **Consecuencias (datos, sin elegir):** A deja 1 impl (`CPIndex`) tras traits sellados; extensiones futuras requieren abrir el sellado (minor con `#[non_exhaustive]` o major si se rompe). B cambia paths públicos de `storage::vfile` (mayor blast radius, posible major). C mantiene el ciclo como deuda documentada (la mudanza Screaming no se puede cotizar).
- **Decisión:** PENDIENTE HUMANO (question abajo; el humano la articula con sus palabras en el ADR final `docs/architecture/adr/NNN_*.md`).

## QUESTION — UNA ronda al humano (Gate D; sin aprobación no hay código)
> **Q-F3X (diseño trait-split):** ¿Apruebas el diseño A (hoja `src/index_port.rs` + traits `IndexPort/MmapBackend/VectorStoreRef` sellados + flag a kernel `NodeFlags::TOMBSTONE`, migraciones X1–X3 mecánicas sin cambio semántico)?
> - **Opción A — GO con este diseño (Recomendado):** apruebas nombre `index_port.rs` + traits del contrato + slices X1→X2→X3. El worker ejecuta F3X-impl. Motivo: valida Q3 (patrón M3) con el ajuste mínimo (flag a kernel existente, trait donde M3-inversión no cabe), aditivo sin major, acyclic verificable por `rg`+`cargo modules`.
> - **Opción B — GO con otro nombre/ubicación:** mismo diseño, hoja distinta (p. ej. `src/storage_port.rs` o `src/kernel/index_api.rs`). Indica el nombre; el worker lo usa sin re-diseño.
> - **Opción C — DEFER explícito:** no se implementa; se registra fila FIND con revisit date (BOUNDARIES §3 opción 2). La mudanza sigue sin cotizar.
> - **(Recomendado): Opción A.**

## 10. VALIDACION+CIERRE (diseño-only)
- Verify diseño: tabla §7 (8/8 con file:línea en `dd892c4c`) + contrato tipado (nombres/firmas/errores/dirección) + slices X1–X3 especificados + plan acyclic/edge-audit + ADR-datos sin decisión + question emitida (arriba / BLOQUEO).
- Verify mecánico corrido (bash solo lectura): `rg` 8 aristas ✅ + `rg storage::vfile` alcance ✅ + `git log` base `dd892c4c` ✅ + `git status` (solo WIP ajeno preexistente + plan untracked; este file es el único artefacto nuevo).
- DoD diseño: (1) verificable por el lead sin ejecutar código, (2) sin scope creep (cero `src/` tocado), (3) este file + Save Point.
- **Gates:** D disparado (question Q-F3X al humano — BLOQUEO hasta respuesta) · V no-disparado (re-medición confirma A1; H1/H2 informativos, misma dirección/dueño) · C: NO commitear (commitea el lead tras F3X-impl; esta tarea solo crea el file).

## Impacto mapeado (Regla 0)
- **Leídos completos:** plan Fase 3 §F3X, BOUNDARIES.md (§3 + BND-02/03/04 + firma A1), C2M3.md (diseño+steps+veredictos), F3G.md (base `dd892c4c` + tabla D), `search_profile.rs:1-50`, `archive.rs:1-30`, `engine/mod.rs:1-60`, `init.rs:1-40`, `maintenance.rs:1-30`, `index/mod.rs:1-40`, `serialize/file.rs:1-20`, `graph/types.rs:1-25,55-119`, `search/layer.rs:1-30`, `flat.rs:1-35`, `graph/core.rs:15-55`, `node/flags.rs:1-40`.
- **Referencias hacia dentro (nuevas):** `docs/tasks/F3X.md` (este file; sin entrantes aún — F3X-impl lo consume).
- **Referencias entrantes (futuras):** F3X-impl (worker X1–X3) + F3C (evita `init.rs`) + cotización mudanza (gate).
- **Veredicto:** diseño-only, reversible (borrar/editar este file), cero impacto runtime. Gate D: GO-con-ajuste pendiente de humano.

## Steps (esta tarea = X0 diseño; X1–X3 especificados arriba para F3X-impl)
### Step X0: diseño arch + ADR-datos + question (ESTA TAREA)
- **Archivos (lectura):** §2 clave+relacionados. **Archivos (escritura):** `docs/tasks/F3X.md` (este file).
- **Acción:** re-medir 8 aristas + H1/H2, validar Q3 (Gate D diseño), contrato tipado, slices X1–X3, ADR-datos, question Q-F3X.
- **Verify:** §10 diseño-only ✅ (este file contiene tabla + contrato + slices + ADR-datos + question).
- **Estado:** ✅ COMPLETE (2026-09-14, sin commit — commitea el lead con F3X-impl o según indique).

## Context Save Point
- **Fecha:** 2026-09-14 · **Branch:** develop · **Base:** `dd892c4c` (F3G) · **Worktree:** limpio salvo WIP ajeno (`.opencode` M, `desktop/src-tauri/Cargo.lock` M) + plan Fase 3 untracked + este file nuevo.
- **Decisiones:** Q3 validada con ajuste (hoja para traits, kernel existente para flag); inversión total rechazada; Box/re-export prohibido como migración; nombre `index_port.rs` propuesta (humano confirma en Q-F3X).
- **Problemas conocidos:** `question` tool no disponible en este runner → ronda emitida como BLOQUEO estructurado; C-SEALED/ISP-DIP [cita NO VERIFICADA — sin red] pendiente para F3X-impl; `cargo modules --acyclic` con timeout conocido en Windows (rg-audit como evidencia primaria en X3).
- **Próxima tarea:** F3X-impl (worker X1→X2→X3) tras respuesta humana a Q-F3X; nextTask del plan: F3C.

## §7. RESULTADO
```
RESULTADO: 🟡 INCOMPLETO (diseño completo; pendiente aprobación humana Gate D)
STEPS_OK: 1/1 (X0 diseño)
PROXIMO_STEP: F3X-impl (worker X1→X2→X3 tras GO humano a Q-F3X)
COMMIT_HASH: ninguno (diseño-only; NO commitear)
ARCHIVOS: docs/tasks/F3X.md
VERIFY_CONTRATO: pasa (diseño-only: tabla 8/8 file:línea en dd892c4c + contrato tipado + slices + ADR-datos + question emitida; bash solo lectura OK; cero src/ tocado)
BLOQUEO: Q-F3X al humano — ¿GO opción A (recomendada), B (otro nombre), o C (DEFER con FIND)? Sin respuesta no hay código (Gate D por diseño).
GATES_EVALUADOS: P:no (ya decidido en plan Gate P.3) D:disparado (Q-F3X emitida; hipótesis Q3 validada con ajuste H2) V:no (re-medición confirma A1; H1/H2 informativos misma dirección) C:no (diseño-only, commitea el lead)
SKILLS_CARGADAS: campaign-executor, progreso, systematic-debugging, code-review-and-quality, doubt-driven-development, source-driven-development, planning-and-task-breakdown, codebase-memory, api-and-interface-design, documentation-and-adrs
```

## F3X-impl (worker) — ejecución 2026-09-14 (base `0afa8181`, GO-humano opción A)

### SDP (phase=BUILD + keywords traits/refactor/ports — 8, 1 línea c/u)
- api-and-interface-design — Contract First del trait/hoja (firmas + errores + dirección). [core]
- test-driven-development — RED (edge-audit + acyclic como repro) → GREEN → verify. [core]
- systematic-debugging — root-cause del muro X1b con evidencia cargo antes de declarar HALLAZGO.
- code-review-and-quality — auto-revisión 5 ejes del diff X1a.
- doubt-driven-development — verificación adversarial (¿la hoja oculta una arista? focus-graph dice no).
- source-driven-development — feature-gates `memmap2` + sellado verificados contra código (sin red: la guía externa C-SEALED queda [cita NO VERIFICADA]).
- planning-and-task-breakdown — slices X1a/X1b + checkpoints.
- codebase-memory — sustituido por `rg` + Read directos (sin campaign MCP en este runner; justificado).
- Base extra cargada (fuera del cap SDP): campaign-executor, progreso, performance-optimization (hot path: mover, no mejorar).

### X1a ✅ — hoja + impls (aditivo, cero cambio semántico)
- NEW `src/index_port.rs` (~70L): `IndexBackendKind` + traits `VectorStoreRef`/`MmapBackend`/`IndexPort`, `pub(crate)` sellados por visibilidad (lectura del GO-humano "sellados pub(crate)"; el contrato proponía `pub` + seal; sin `serde` — import sin uso; sin `#[non_exhaustive]` — innecesario con `pub(crate)`).
- NEW `src/index/port_impl.rs` (~110L): `impl VectorStoreRef for File`, `impl MmapBackend for IndexBackend` (+ `for CPIndex` delegando), `impl IndexPort for CPIndex` (`open_index`/`rebuild_from_bytes`/`fresh_like` byte-idéntico a `fresh_index_like`).
- EDIT `src/lib.rs` (+1: `pub(crate) mod index_port;`), `src/index/mod.rs` (+1: `mod port_impl;`).
- Deuda TEMP explícita: `#![allow(dead_code)]` en los 2 files nuevos (se retira en X3 al migrar el primer consumidor; grep `TEMP(F3X-X1)`).
- Verify X1a: `cargo check -p vantadb --tests --all-targets` ✅ · `cargo clippy -p vantadb --all-targets -- -D warnings` ✅ · `cargo fmt --check -p vantadb` ✅ · focus-graph `index_port` → solo aristas a `std`/`core`, cero a `index`/`storage` ✅ (neutralidad probada).
- Auto-revisión 5 ejes: correctness (impls espejan lógica inherente) / readability (TEMP marcado) / architecture (solo aristas descendentes) / security (N/A, sin unsafe/input) / performance (Regla 9: nada que medir — código aún no llamado).

### X1b 🟡 BLOQUEADO — HALLAZGO-H4 (muro mecánico, no re-diseño en silencio)
**HALLAZGO-H4: el contrato exige a la vez (i) `rg use crate::index::(CPIndex|IndexBackend) src/storage/ → 0` + acyclic sin aristas storage↔index, y (ii) "NO cambia sus firmas / aditivo minor". Con granularidad item-level de `cargo modules` (probada: `--acyclic` falla en `GraphAccumulator → GraphAccumulator::new`, i.e. tipo→fn), ambas son conjuntamente insatisfacibles:**
- `src/storage/engine/mod.rs:325` `pub hnsw: ArcSwap<CPIndex>` + `:441` `pub vec_index() -> Guard<Arc<CPIndex>>` + `:385` `replay_write_node(... hnsw: &CPIndex ...)` — quitar el `use` rompe el tipo del campo/firmas (incl. API pública).
- `src/storage/archive.rs:42,173,199` `pub fn (compact_layout|traverse_graph|reindex_nodes)(... &CPIndex ...)` + `:208,221,230` (`pub(crate)` fresh/rebuild) — idem; el focus-graph lista aristas fn-level (`:3080,3083,3088,3095,3101,3104`) que sobreviven a cualquier migración solo de `use`.
- `src/storage/engine/init.rs:300-304` (`init_indexes -> ...(CPIndex,...)`), `:378` (`recover_state(... &mut CPIndex ...)`); `maintenance.rs` A4 + asignación `.backend` (`:198`).
- Lado B: `src/index/search/nearest.rs:19` `pub search_nearest(... Option<&File>)` + firmas FQ H1 (`diskann.rs:400`, `ivf.rs:424`, `scann.rs:239`, `nearest.rs:26,52`, `search/mod.rs:21`, `layer.rs:27`, `flat.rs:98`) + `VecIndex::search` (`pub(crate)` — esa SÍ es migrable sin superficie).
- Regla §8 del propio diseño: "si el worker detecta un `pub` que deba cambiar → ADR + semver major (precedente ADR-041)". Detectados 6 (`traverse_graph`, `compact_layout`, `reindex_nodes`, `hnsw`, `vec_index`, `search_nearest`) en crate v0.5.0 → corresponde ADR + decisión humana, NO migración silenciosa.
- Evidencia: `cargo modules dependencies --lib -p vantadb --acyclic` → falla-rápido solo en el artefacto accumulator (BND-03X exceptuado por F3G); el gate no distingue storage↔index mientras el artefacto exista → evidencia primaria = focus-graph + `rg` (tal como anticipa el diseño §X3).
- HALLAZGO-H3 (benigno): `src/storage/vfile.rs:46,858` es `#[cfg(test)]` — fuera de `--lib`, no requiere migración.

### Gate V — question al humano (una ronda; sin respuesta no hay más código)
> **Q-F3X-impl (muro X1b):** la migración exige cambiar 6 firmas `pub` (lista arriba) o el ciclo persiste a nivel item. ¿Cómo sigo?
> - **Opción A — GO con ADR + major (recomendado):** el humano firma ADR (Regla 5) aceptando el cambio de firmas `pub` → worker ejecuta X1b/X2/X3 completos (sigs a `dyn`/`impl Trait` + TEMP-debt fuera + suites + edge-audit). Costo: superficie pública rota documentada.
> - **Opción B — GO parcial sin tocar `pub`:** migrar solo lo `pub(crate)`/privado (B4 flag→kernel, `VecIndex`, `file.rs`/`types.rs` internos) + hoja en uso parcial; el ciclo item-level PERSISTE en las 6 firmas `pub` → contrato acyclic queda en rojo documentado como deuda con FIND + revisit date.
> - **Opción C — DEFER explícito:** no más código; fila FIND con revisit date (BOUNDARIES §3 opción 2); la hoja X1a queda como base reutilizable o se revierte (2 files nuevos + 2 líneas mod).
> - **(Recomendado): Opción A.**

### Estado impl
- X1a ✅ (en worktree, SIN commit — commitea el lead) · X1b 🟡 (bloqueado en Q-F3X-impl) · X2/X3 ⬜ (pendientes de la respuesta).
- Worktree esperado: `M src/lib.rs`, `M src/index/mod.rs`, `?? src/index_port.rs`, `?? src/index/port_impl.rs` (+ este file).
- nextTask: F3C sigue en cola tras la resolución de Q-F3X-impl.

## F3X-impl — ejecución Opción A (2026-09-14, ADR-042) — COMPLETO ✅

### X1b ✅ — migración storage→index (full-dyn, 6 firmas pub + internos)
- `StorageEngine.hnsw: ArcSwap<Box<dyn IndexPort>>` (Box: `Arc` exige `Sized` para `RefCnt`; verificado contra fuente vendored arc-swap 1.9.2) + `vec_index() -> Guard<Arc<Box<dyn>>>` + `replay_write_node(&dyn)` + `FLAG_TOMBSTONE` alias a kernel.
- `archive.rs`: 5 sigs a `&dyn/&mut dyn`, cuerpos vía trait, `fresh_index_like` → shim `#[cfg(test)]` (tests intactos), `random_level`, `add_node*`.
- `init.rs`: factories `port_impl::{open,new_in_memory,new_mmap}` (Box único; `Arc::new` solo en stores), `set_flat_threshold`, `recover_state(&mut dyn)`, WAL paths vía trait.
- `maintenance.rs`: dance `save_vector_index` → `persist_mmap` (movido byte-idéntico a port_impl, RCU), rebuild vía `fresh_box` + archive, scans vía `scan_entries`/`all_node_ids`, vacuum/merge/quantize/fresh (`repair_orphan_links`) vía trait, `release_mmap_vector` MOVIDO a `vfile.rs` (único caller).
- `delete/get/insert/ops/txn/stats`: swaps mecánicos (`storage_offset_of`, `remove_node`, `add_node*`, `node_count`, `entry_point*`, `is_sq8_vector`, `stored_vector`/`node_view` en rescates legacy con lookup único).
- `cache_warmer::hnsw_top_layer_ids(&dyn)` + `cost_estimator` (getters `index_kind/flat_threshold/node_count`) + `sdk/api` (`distance_metric`) + `sdk/vector` (sin cambio: `IndexPort::search` vía dyn) + `physical_plan` (`Some(&*vs)`).
- Patrón `&***/&*` documentado (Guard→Arc→Box→dyn vs Box→dyn); llamadas a métodos `dyn` NO requieren import (probado: layer.rs); sigs sí.
- Desviaciones forzadas del contrato (compilador/grafo, no gusto): D1 traits `pub` (benches externos llaman métodos dyn); D2 caídos `IndexBackendKind/resident_bytes/Snapshot/fresh_like(Sized)` (fábricas los superan) + ~25 métodos vivos por censo; D3 fábricas free fns (dyn no llama statics `Sized`); D4 `FreshHnswReport`+`IndexType` movidos a hoja (re-exports BND-04, sigs intactos); D5 `release_mmap_vector` a storage; D6 Box+`Arc::new` en stores; D7 7º cambio pub-clase: path de `release_mmap_vector` (flaggeado por semver-checks; misma clase ADR-042); D8 `MmapMut` cfg re-escrito a path canónico `vfile_mmap`; D9 iteración snapshot≡live (todo bajo `insert_lock`); D10 pairing `total_nodes` preservado (solo vacuum decrementa, comentado); D11 una indirección Box extra por load (ruido vs DashMap; red F3B).

### X2 ✅ — migración index→storage
- B4a/B4b + `search/tests.rs` → `NodeFlags::TOMBSTONE` (kernel existente); `VecIndex::search` + 8 sigs → `Option<&dyn VectorStoreRef>`; `layer.rs` vía trait (`read_header`+`mmap_bytes` añadidos tras hallazgo empírico E0599); `file.rs`/`types.rs` MmapMut al path canónico (cfg-fantasma bajo default); `search_nearest/with_metric` con dyn.
- Ventana residual diseño-bendecida: `graph/types.rs` (`Mmap::map`, `get_resident_bytes` ×2, body-FQ invisible a cargo-modules) — unidireccional, sin ciclo.

### X3 ✅ — verify contrato + cierre
- `cargo check -p vantadb --tests --all-targets` ✅ · `clippy --all-targets -D warnings` ✅ · `fmt --check` ✅ (vía `cargo fmt`).
- `nextest -p vantadb --lib storage index --build-jobs 2`: **783 passed, 0 failed** (incl. roundtrips mmap, rebuild idempotente, vacuum, quantize, recall parity, concurrent 130s) → sin cambio semántico probado.
- `cargo modules --acyclic`: SOLO artefacto accumulator (BND-03X exceptuado F3G) ✅ · focus-graphs: cero aristas storage↔index en ambas direcciones; storage→leaf e index→leaf según diseño ✅.
- Edge-audit rg: P1 scoped 0 (2× `cfg(test)` archive-shim/tests); P2 0; P3/P6 test-only (`search/tests.rs` es `#[cfg(test)]`, fixtures necesitan `File` concreto); P4 3× cfg-gated `vfile_mmap`; P5 ventana types.rs ×3; H1 7/7 migradas; FLAG alias ✅.
- `cargo semver-checks -p vantadb --baseline-rev 0afa8181` (0.49.0, ~14min): **195 pass / 1 fail** — solo `function_missing: index::release_mmap_vector` (movido, D7). Las 6 firmas ADR-042 no las flaggea la tool (cobertura); el major va por ADR-042 + release-plz del lead. Rojo major = esperado y documentado.
- TEMP `allow(dead_code)`: retirado orgánicamente (todo el surface tiene callers; clippy -D lo prueba; `rg TEMP(F3X` = 0).
- `vanta-review`: SIN runner disponible → **visa pendiente del lead** (diff + este file + ADR-042 como paquete de revisión).
- Gates: P:no (GO-A vigente) · D:no (diseño+ADR-042) · V:resuelto-ejecutado (opción A) · C:no (commitea el lead).
- Worktree final: 41 archivos (lista en `git status`); SIN commit. nextTask: **F3C**.

## F3X-impl — post-review vanta-review (CHANGES-REQUIRED atendido, SIN commit)

- **H1-ALTA ✅ (bloqueante, TDD RED→GREEN):** `insert_hnsw_leveled` colapsaba niveles (RNG fresco por llamada → primer draw constante). RED: test extendido `d1a_insert_hnsw_leveled_indexes` (bulk 200 nodos, asevera `>1` nivel distinto vía `node_layers`) **falló pre-fix con `got {1}`**. Fix: `bulk_levels_seeded(seed, n) -> Vec<usize>` en trait (un StdRng, n draws — stream idéntico al loop pre-split) + loop con `zip` precomputado; `random_level_seeded` (1-draw) ELIMINADO del trait+impl (quedaba sin callers → clippy lo habría flaggeado). GREEN: test pasa en suite + full suite debajo. Claim "sin cambio semántico" restaurado por evidencia.
- **H2-MEDIA ✅:** vacuum un solo Guard (`let hnsw = self.hnsw.load();` una vez, reutilizado en scan + removal). Sin `insert_lock` (cambio de concurrencia no pedido, no hecho).
- **H3-MEDIA ✅:** traits sellados con `#[doc(hidden)] pub mod sealed` + bounds en los 3 traits + impls (`port_impl.rs` ×2, `vfile.rs` ×1). Nota mecánica: `mod private` literal es inimplementable (los impls deben vivir cross-module + E0365) y `pub(crate) mod` tripea `private_bounds` bajo `-D warnings`; la forma doc-hidden es el sello viable (llamar ✅, implementar fuera solo nombrando hidden = unsupported). clippy -D verde lo prueba.
- **H4 ✅ (pre-cumplido):** `IndexBackendKind` ya no existe (borrado en S0a/D2 al quedar superado por fábricas); `rg IndexBackendKind` = 0 en todo el repo. Sin acción.
- **B2:** hecho por el lead (7º item en ADR-042) — no tocado aquí.
- **B1 ✅:** precisión §X3: evidencia = nivel `use` (P1 scoped-0, P2 0, P3/P6 test-only, P4 cfg-ghosts) + focus-graph (cero aristas cruzadas ambas direcciones); residual aceptado diseño-bendecido listado explícito = `graph/types.rs:74,89,99` (`get_resident_bytes` ×2, `Mmap::map` ×1; body-FQ invisible a cargo-modules, unidireccional).
- **B3 ✅:** comentario `// &***: Guard → Arc → Box → dyn (one deref per wrapper).` en los 4 sitios (maintenance.rs:501,505,511 — numeración pre-review; get.rs:399).
- **B4 (anotado, no renombrar):** en futuros splits, no repetir patrón stutter (`fresh_index_like`→`fresh_like`→`fresh_box` convivieron); nombrar 1 sola vez desde el diseño.
- **Re-verify mínimo:** `cargo check -p vantadb --tests --all-targets` ✅ · `cargo clippy -p vantadb --all-targets -- -D warnings` ✅ · `cargo fmt --check -p vantadb` ✅ · `cargo nextest run -p vantadb --lib storage index --build-jobs 2` → **783 passed, 0 failed** (incl. H1 GREEN + vacuum/quantize/rebuild/roundtrips/recall-parity/concurrent) ✅. SIN commit (commitea el lead con visa final).

> **Cierre 2026-09-14:** COMPLETED (diseno A + impl X1-X3 + review changes-required atendido 7/7 + 783/783 + semver major esperado; commits 0afa8181 diseno + 13f0f729 codigo, ADR-042).
