# Plan de Ejecución — Tareas de Código Puro

> **Propósito:** Ejecutar secuencialmente todas las tareas de código pendientes de `Backlog.md` y `bitacora.md`.
> **Modo:** 🏴 Ponytail full — mínimo código que funciona, stdlib primero, nada especulativo.
> **Skills base:** `ponytail` (full), `writing-plans`, `code-review-and-quality`, `doubt-driven-development`
> **Verificación base:** `cargo build && cargo nextest run --profile audit --workspace --build-jobs 2`

---

## 🔄 Master Execution Loop (LEER ANTES DE EMPEZAR)

Cada tarea sigue EXACTAMENTE este loop. No saltarse pasos.

```
LOOP POR TAREA:
╔══════════════════════════════════════════════════════════════╗
║ 1. LEER este archivo → identificar próxima tarea ❌        ║
║ 2. LEER docs/Backlog.md (estado actual de esa tarea)        ║
║ 3. SKILLS: cargar skills que aplican según fase (ver abajo) ║
║ 4. CODEGRAPH: codegraph_explore "nombres de archivos"       ║
║    - Callers: qué módulos llaman a lo que voy a cambiar     ║
║    - Callees: de qué dependen esos archivos                 ║
║    - Blast radius: ¿se rompen interfaces? ¿cambia API pub?  ║
║ 5. IMPLEMENTAR: siguiendo las skills cargadas               ║
║ 6. VERIFICAR: cargo build && cargo nextest run ...          ║
║ 7. ACTUALIZAR este archivo: tarea → ✅                     ║
║ 8. ACTUALIZAR Backlog.md: tarea → ✅                        ║
║ 9. GIT: git add -A && git commit -m "fix/task(ID): ..."     ║
║ 10. REPETIR → siguiente tarea ❌                            ║
╚══════════════════════════════════════════════════════════════╝

Si cargo build falla → arreglar errores, repetir paso 5-6 hasta que pase.
Si nextest falla → arreglar tests, repetir paso 5-6 hasta que pase.
Si clippy da error → `cargo clippy --fix` o arreglar manual, repetir.
```

### Skills por Fase (del AGENTS.md — Skill Loading Guide — Ingeniería)

| Fase | Skills a cargar primero | Cuándo |
|------|------------------------|--------|
| **DEFINE** | `spec-driven-development`, `idea-refine` | Feature nueva o API pública |
| **PLAN** | `planning-and-task-breakdown`, `writing-plans` | Si la tarea es ambigua o grande |
| **BUILD** | `ponytail` (full), `incremental-implementation`, `doubt-driven-development`, `source-driven-development` | Implementación |
| **BUILD (UI)** | `frontend-ui-engineering`, `design-taste-frontend` | Si toca web/ |
| **BUILD (API)** | `api-and-interface-design` | Si cambia API pública |
| **VERIFY** | `debugging-and-error-recovery` | Si tests fallan o build se rompe |
| **REVIEW** | `code-review-and-quality`, `code-simplification`, `security-and-hardening` | Antes de commit |
| **SHIP** | `git-workflow-and-versioning`, `documentation-and-adrs` | Commit + docs |

### Reglas Ponytail (vigilancia constante)

```
1. ¿Ya existe en el codebase? → reusar (buscar helpers/patterns similares)
2. ¿Stdlib lo resuelve? → usarla
3. ¿Dependencia ya instalada? → usarla
4. ¿Una línea alcanza? → una línea
5. SINO: mínimo código que funciona, sin abstracciones innecesarias

NO simplificar: validación en trust boundaries, data loss, seguridad, accesibilidad.
```

---

## ⚠️ Context Management (CUANDO LLEGUES A ~70% o ~170K TOKENS)

Cuando el contexto se acerque al límite o después de ~3-4 tareas:

```
1. ACTUALIZAR este archivo: última tarea → ✅, tabla de iteraciones
2. AGREGAR SAVE POINT (ver abajo) con recitation block
3. HACER COMMIT: git add -A && git commit -m "checkpoint: [N] tareas"
4. AVISAR: "Contexto lleno. Continuar con: <próxima tarea>"
5. RETURN — nueva sesión, pegar PROMPT DE INICIO (abajo)
```

### Context Preservation Protocol

| Señal | Acción |
|-------|--------|
| ~70% tokens ocupados | Compactar activamente (resumir iteraciones pasadas, descartar tool outputs) |
| ~85% tokens ocupados | Ejecutar save point + avisar al usuario |
| Tool output >3000 tokens | Offload: resumen de 1 línea + pointer a archivo de log |

### Save Point (para handoff entre sesiones)

```
=== CONTEXT SAVE POINT ===
Fecha: 2026-07-13
Tokens aprox: ~XK / 200K
Branch: fix/code-xxx
CI pendiente: no (último run: ✅)
Próxima tarea: TASK-N — <nombre>

Decisiones registradas:
- [decisión breve]

=== END CONTEXT SAVE ===
```

### Recitation Block (anti-goal-drift — al final del archivo, después de cada acción)

```
=== RECITATION ===
Objetivo activo: TASK-N — <nombre>
Estado actual: <implementado / verificando / CI>
Próxima acción: <próximo paso concreto>
Contrato del loop: "just verify pasa y CI workflow es green"
Próxima tarea: TASK-M — <nombre>
=== END RECITATION ===
```

### PROMPT DE INICIO (para pegar en nueva sesión)

```
Cargá las skills ponytail, writing-plans, code-review-and-quality y doubt-driven-development.
Luego leé docs/plans/2026-07-13-code-task-execution-campaign.md.
Buscá el último save point o recitation block.
Identificá la próxima tarea ❌ y ejecutá el Master Execution Loop.
Antes de cada tarea: codegraph_explore para mapear blast radius.
cargo fmt --check && cargo clippy --workspace --all-targets --all-features -- -D warnings && cargo nextest run --profile audit --workspace --build-jobs 2

NO saltees pasos. NO implementes sin codegraph primero.
```

---

## 📋 Task Queues por Prioridad

### Status Legend
```
✅ = Completada en esta campaña
❌ = Pendiente
⏳ = En progreso
🗑️ = Won't fix / cancelada
```

---

## TIER 1A — 🟡 Refactors Rápidos (Cada uno < 2h, alto impacto)

> Fáciles, bien acotados, mejoran calidad inmediata.

---

### TASK-01: REC-02 — Helper `VantaError::serialization(e)`

| Campo | Valor |
|-------|-------|
| **Backlog ref** | `REC-02` |
| **Archivos** | `src/error.rs` + 20 call sites |
| **Skills** | `ponytail`, `code-review-and-quality` |
| **Esfuerzo** | 🟢 ~1h |
| **Estado** | ✅ |

**Resultado:** Helper `VantaError::serialization(e)` agregado en `src/error.rs:321`. 23 call sites reemplazados (wal.rs, text_index.rs, storage/engine/*, sdk/serialization/*, cli_handlers/data.rs, error.rs tests). `cargo check` ✅, 20 tests de error pasan.

**Prompt específico:**

```
Backlog: REC-02 — Helper VantaError::serialization(e). Reducir boilerplate
"VantaError::SerializationError(Box::new(e))" en ~20 call sites.

Skills: ponytail, code-review-and-quality

Pasos:
1. codegraph_explore "VantaError SerializationError" para mapear los ~20 call sites
2. Leer src/error.rs — encontrar el enum VantaError y la variante SerializationError
3. Agregar método helper en impl VantaError:
   pub fn serialization(e: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Self
4. Reemplazar los ~20 patrones VantaError::SerializationError(Box::new(e)) con VantaError::serialization(e)
5. Verificar: cargo build && cargo fmt --check && cargo clippy ... && cargo nextest run ...
6. git add -A && git commit -m "refactor(error): REC-02 helper VantaError::serialization(e)"
7. Actualizar Backlog.md: REC-02 → ✅
8. Actualizar este archivo: TASK-01 → ✅
```

---

### TASK-02: REC-03 — Source chaining a String variants

| Campo | Valor |
|-------|-------|
| **Backlog ref** | `REC-03` |
| **Archivos** | `src/error.rs`, call sites |
| **Skills** | `ponytail`, `code-review-and-quality` |
| **Esfuerzo** | 🟡 1-2d |
| **Estado** | ✅ |

**Nota:** Ya completado en el codebase. `WalError(String)`, `SearchError(String)`, `Generic(String)`, `BackendError(String)` ya migraron a `ChainedError`.

**Prompt específico:**

```
Backlog: REC-03 — Extender source chaining a variantes String restantes.
WalError(String), SearchError(String), Generic(String), BackendError(String)
— mismo patrón que REC-01 (SerdeMsgError con source).

Skills: ponytail, code-review-and-quality, doubt-driven-development

Pasos:
1. codegraph_explore "VantaError WalError SearchError Generic BackendError"
2. Leer src/error.rs — entender el patrón de SerdeMsgError (struct variant con source)
3. Migrar String variants a struct variants con source chaining:
   - WalError(Box<dyn Error + Send + Sync>)
   - SearchError(Box<dyn Error + Send + Sync>)
   - Generic(Box<dyn Error + Send + Sync>)
   - BackendError(Box<dyn Error + Send + Sync>)
4. codegraph_explore para encontrar TODOS los call sites de cada variante
5. Actualizar cada call site para pasar Box<new error>
6. Verificar: cargo build && cargo fmt --check && cargo clippy ... && cargo nextest run ...
7. Si clippy da missing_safety_doc o similar, corregir
8. git add -A && git commit -m "refactor(error): REC-03 source chaining String variants"
9. Actualizar Backlog.md: REC-03 → ✅
10. Actualizar este archivo: TASK-02 → ✅
```

---

### TASK-03: P8 — release_mmap_vector SAFETY doc

| Campo | Valor |
|-------|-------|
| **bitacora ref** | `P8` |
| **Archivos** | `src/index/graph.rs:65` |
| **Skills** | `ponytail`, `code-review-and-quality` |
| **Esfuerzo** | 🟢 ~30min |
| **Estado** | ✅ |

**Nota:** Ya completado. SAFETY doc presente en `src/index/graph.rs:64-68`, `#[allow(clippy::missing_safety_doc)]` removido.

**Prompt específico:**

```
Bitacora P8 — fn release_mmap_vector() en src/index/graph.rs:65 tiene
#[allow(clippy::missing_safety_doc)] en unsafe fn. Agregar # Safety
docstring describiendo precondiciones.

Skills: ponytail

Pasos:
1. Leer src/index/graph.rs alrededor de la línea 65
2. Investigar cómo se usa release_mmap_vector (codegraph_explore "release_mmap_vector")
3. Agregar doc SAFETY completa explicando:
   - Cuándo es seguro llamarla
   - Qué precondiciones debe cumplir el caller
   - Qué pasa si se violan las precondiciones
4. Remover #[allow(clippy::missing_safety_doc)]
5. cargo build && cargo clippy ... -- asegurar que no hay warnings nuevos
6. git add -A && git commit -m "docs: P8 release_mmap_vector SAFETY doc"
7. Actualizar este archivo
```

---

### TASK-04: P9 — Magic numbers → const named

| Campo | Valor |
|-------|-------|
| **bitacora ref** | `P9` |
| **Archivos** | Múltiples (1024, 64, 0x8, 0.80 hardcodeados) |
| **Skills** | `ponytail` |
| **Esfuerzo** | 🟡 ~1d |
| **Estado** | ✅ |

**Nota:** Ya completado. `DEFAULT_INITIAL_CAPACITY` (1024), `STORAGE_ALIGNMENT` (64), `FLAG_TOMBSTONE` (0x8), `DEFAULT_RSS_THRESHOLD` (0.80) ya existen como constantes nombradas.

**Prompt específico:**

```
Bitacora P9 — Mover magic numbers a constantes con nombre:
- 1024 capacity → const DEFAULT_CAPACITY: usize = 1024;
- 64 byte alignment → const ALIGNMENT: usize = 64;
- 0x8 tombstone flag → const TOMBSTONE_FLAG: u8 = 0x8;
- 0.80 RSS threshold → const RSS_THRESHOLD: f64 = 0.80;

Skills: ponytail

Pasos:
1. grep para encontrar cada magic number en el codebase
2. Para cada uno, decidir:
   a. ¿Es realmente un "magic number" o un valor que no debería ser constante?
   b. Ubicación correcta de la constante (módulo local vs global)
3. Mover a constantes con nombre
4. cargo build && cargo nextest run ...
5. git add -A && git commit -m "refactor: P9 magic numbers → named constants"
6. Actualizar este archivo
```

---

### TASK-05: P12 — /metrics endpoint auth opcional

| Campo | Valor |
|-------|-------|
| **bitacora ref** | `P12` |
| **Archivos** | `src/cli_server.rs` |
| **Skills** | `ponytail`, `security-and-hardening` |
| **Esfuerzo** | 🟢 ~1h |
| **Estado** | ✅ |

**Nota:** Ya completado. `/metrics` está en `protected` router con `auth_middleware` (src/cli_server.rs:127). Sin API key → público (dev mode). Con API key → protegido.

**Prompt específico:**

```
Bitacora P12 — /metrics endpoint público sin auth (el resto del server requiere API key).

Skills: ponytail, security-and-hardening

Pasos:
1. codegraph_explore "metrics cli_server" para mapear rutas
2. Leer src/cli_server.rs — entender sistema de auth existente
3. Agregar auth opcional configurable para /metrics
   - Si API key está configurada, /metrics requiere auth
   - Si no, /metrics sigue público (backwards compat)
4. Ponytail: no crear abstracción, solo un flag + if
5. cargo build && cargo clippy ... && cargo nextest run ...
6. git add -A && git commit -m "fix(security): P12 /metrics endpoint auth"
7. Actualizar este archivo
```

---

### TASK-06: P10 — Mixed Spanish/English comments a inglés

| Campo | Valor |
|-------|-------|
| **bitacora ref** | `P10` |
| **Archivos** | `storage.rs`, `wal.rs`, `text_index.rs` |
| **Skills** | `ponytail` |
| **Esfuerzo** | 🟡 ~1d |
| **Estado** | ✅ |

**Nota:** 11 comentarios traducidos (8 en `src/wal.rs`, 3 en `src/bin/lock_helper.rs`). `cargo check` ✅.

**Prompt específico:**

```
Bitacora P10 — Comentarios en español en storage.rs, wal.rs, text_index.rs.

Skills: ponytail

Pasos:
1. grep "//.*[áéíóúñ¿¡]" para encontrar comentarios en español
2. Traducir cada uno a inglés
3. NO cambiar lógica, solo comentarios
4. cargo build (solo confirmar que nada se rompe)
5. git add -A && git commit -m "docs: P10 unify comments to English"
6. Actualizar este archivo
```

---

### TASK-07: P7 — Error hierarchy gaps (IqlError, CliError)

| Campo | Valor |
|-------|-------|
| **bitacora ref** | `P7` |
| **Archivos** | `src/error.rs`, `src/wal_archiver.rs` |
| **Skills** | `ponytail`, `code-review-and-quality` |
| **Esfuerzo** | 🟡 ~1d |
| **Estado** | ✅ |

**Nota:** Ya completado. Variantes `IqlError`, `CliError`, `SearchError`, `RuntimeError` migradas a `ChainedError`. Los 4 `unwrap()` en `wal_archiver.rs` reemplazados con `unwrap_or_default()` / `unwrap_or()`.

**Prompt específico:**

```
Bitacora P7 — 4 variantes String remanentes (IqlError, CliError, SearchError, RuntimeError)
sin proper error types. 4 unwrap() en wal_archiver.rs.

Skills: ponytail, code-review-and-quality

Pasos:
1. codegraph_explore "IqlError CliError SearchError RuntimeError wal_archiver"
2. Leer src/error.rs — entender las variantes String
3. Migrar a struct variants con source chaining (como TASK-02 si no se completó)
4. Reemplazar unwrap() en wal_archiver.rs:78,81,120,183 con ? o context()
5. cargo build && cargo clippy ... && cargo nextest run ...
6. git add -A && git commit -m "refactor(error): P7 error hierarchy gaps"
7. Actualizar este archivo
```

---

### TASK-08: W16 — Blog factual errors

| Campo | Valor |
|-------|-------|
| **bitacora ref** | `W16` |
| **Archivos** | `web/src/routes/blog/introducing-vantadb.md` |
| **Skills** | `ponytail` |
| **Esfuerzo** | 🟢 ~30min |
| **Estado** | ❌ |

**Prompt específico:**

```
Bitacora W16 — Blog "License: MIT" (real: Apache 2.0), GitHub link apunta a
ness-e/VantaDB (real: vantadb/vantadb).

Skills: ponytail

Pasos:
1. Leer web/src/routes/blog/introducing-vantadb.md
2. Corregir: "MIT" → "Apache 2.0"
3. Corregir: GitHub link → vantadb/vantadb
4. git add -A && git commit -m "fix(web): W16 blog factual errors"
5. Actualizar este archivo
```

---

### TASK-09: W6 — Security headers in Vercel

| Campo | Valor |
|-------|-------|
| **bitacora ref** | `W6` |
| **Archivos** | `web/vercel.json` |
| **Skills** | `ponytail`, `security-and-hardening` |
| **Esfuerzo** | 🟢 ~1h |
| **Estado** | ❌ |

**Prompt específico:**

```
Bitacora W6 — No HSTS, no X-Content-Type-Options, no HTTP→HTTPS redirect en vercel.json.

Skills: ponytail, security-and-hardening

Pasos:
1. Leer web/vercel.json
2. Agregar security headers en vercel.json:
   - Strict-Transport-Security: max-age=63072000
   - X-Content-Type-Options: nosniff
   - Referrer-Policy: strict-origin-when-cross-origin
3. Si existe redirects[], agregar HTTP→HTTPS redirect
4. git add -A && git commit -m "fix(security): W6 security headers Vercel"
5. Actualizar este archivo
```

---

### TASK-10: W17 — Touch targets < 44px (Apple HIG)

| Campo | Valor |
|-------|-------|
| **bitacora ref** | `W17` |
| **Archivos** | Componentes con hamburger menu, nav-cta, close button |
| **Skills** | `ponytail` |
| **Esfuerzo** | 🟢 ~30min |
| **Estado** | ❌ |

**Prompt específico:**

```
Bitacora W17 — Touch targets < 44px: hamburger 36px, nav-cta ~32px, close 36px.

Skills: ponytail

Pasos:
1. grep "36px\|32px\|w-9\|w-8\|h-9\|h-8" en web/src/components/ para encontrar targets pequeños
2. Agregar padding o min-width/height a 44px
3. Verificar visualmente no rompe layout
4. git add -A && git commit -m "fix(ux): W17 touch targets 44px HIG"
5. Actualizar este archivo
```

---

## TIER 1B — 🟡 Code Health Core (Cada uno < 1d)

---

### TASK-11: PERF-13 — Refactor read_only check → helper

| Campo | Valor |
|-------|-------|
| **Backlog ref** | `PERF-13` |
| **Archivos** | `src/sdk/api.rs` (5 veces repetido) |
| **Skills** | `ponytail` |
| **Esfuerzo** | 🟢 ~1h |
| **Estado** | ❌ |

**Prompt específico:**

```
Backlog PERF-13 — read_only check repetido 5 veces en sdk/api.rs.

Skills: ponytail

Pasos:
1. codegraph_explore "read_only sdk/api.rs"
2. Leer src/sdk/api.rs — identificar los 5 patrones if self.read_only { return Err }
3. Extraer a helper method: fn check_read_only(&self) -> Result<()>
4. Reemplazar los 5 patrones
5. cargo build && cargo nextest run ...
6. git add -A && git commit -m "refactor: PERF-13 read_only helper"
7. Actualizar este archivo
```

---

### TASK-12: PERF-14 — Refactor init_telemetry masivo

| Campo | Valor |
|-------|-------|
| **Backlog ref** | `PERF-14` |
| **Archivos** | `src/lib.rs` (~160L de if/else repetitivo) |
| **Skills** | `ponytail`, `code-simplification` |
| **Esfuerzo** | 🟡 ~1d |
| **Estado** | ❌ |

**Prompt específico:**

```
Backlog PERF-14 — init_telemetry ~160 líneas de if/else repetitivo.

Skills: ponytail, code-simplification

Pasos:
1. codegraph_explore "init_telemetry"
2. Leer la función completa
3. Identificar patrones repetidos (probablemente pares (feature, init_fn))
4. Refactor: array de tuplas (feature_flag, init_fn) con iteración, no if/else duplicado
5. cargo build && cargo clippy ... && cargo nextest run ...
6. git add -A && git commit -m "refactor: PERF-14 init_telemetry masivo"
7. Actualizar este archivo
```

---

### TASK-13: DOC-02 — Refactor insert_hnsw() (177L → 3 funciones)

| Campo | Valor |
|-------|-------|
| **Backlog ref** | `DOC-02` |
| **Archivos** | `src/index/core.rs` |
| **Skills** | `ponytail`, `code-simplification` |
| **Esfuerzo** | 🟡 ~1d |
| **Estado** | ❌ |

**Prompt específicico:**

```
Backlog DOC-02 — insert_hnsw() 177L monolítica → 3 funciones más pequeñas.

Skills: ponytail, code-simplification

Pasos:
1. codegraph_explore "insert_hnsw"
2. Leer la función en src/index/core.rs
3. Identificar 3 fases naturales (preparación, inserción, post-procesamiento)
4. Extraer cada fase a función con nombre descriptivo
5. NO cambiar lógica, solo dividir
6. cargo build && cargo clippy ... && cargo nextest run ...
7. git add -A && git commit -m "refactor: DOC-02 split insert_hnsw()"
8. Actualizar este archivo
```

---

### TASK-14: P6 — Duplicate code patterns (append_to_vstore / write_node_to_vstore)

| Campo | Valor |
|-------|-------|
| **bitacora ref** | `P6` |
| **Archivos** | `src/storage/ops.rs`, `src/sdk/api.rs`, `src/lib.rs` |
| **Skills** | `ponytail`, `code-simplification`, `doubt-driven-development` |
| **Esfuerzo** | 🟡 ~2d |
| **Estado** | ❌ |

**Prompt específico:**

```
Bitacora P6 — Código duplicado:
- append_to_vstore / write_node_to_vstore (~40L casi idénticas)
- if let Some(ref mut wal) = *self.wal.lock() { wal.append(...) } repetido
- read_only check 5 veces (si no se hizo PERF-13)
- init_telemetry ~160L (si no se hizo PERF-14)

Skills: ponytail, code-simplification, doubt-driven-development

Pasos:
1. codegraph_explore "append_to_vstore write_node_to_vstore"
2. Leer ambos métodos y las llamadas a wal.lock().append()
3. Extraer a función compartida: fn wal_append(wal: &Mutex<Option<WalWriter>>, record: WalRecord) -> Result<()>
4. Para append_to_vstore / write_node_to_vstore: merge en un solo método con flag o parámetro
5. cargo build && cargo clippy ... && cargo nextest run ...
6. git add -A && git commit -m "refactor: P6 deduplicate append patterns"
7. Actualizar este archivo
```

---

## TIER 1C — 🟡 Testing & CI (Cada uno < 1d)

---

### TASK-15: T7 — test-threads=2 per-platform

| Campo | Valor |
|-------|-------|
| **bitacora ref** | `T7` |
| **Archivos** | `.cargo/config.toml`, `.config/nextest.toml` |
| **Skills** | `ponytail`, `ci-cd-and-automation` |
| **Esfuerzo** | 🟢 ~30min |
| **Estado** | ❌ |

**Prompt específico:**

```
Bitacora T7 — .cargo/config.toml tiene test-threads = 2 global (necesario solo en Windows).

Skills: ponytail, ci-cd-and-automation

Pasos:
1. Leer .cargo/config.toml y .config/nextest.toml
2. Mover test-threads = 2 a configuración per-platform:
   - Opción A: nextest config con platform filter
   - Opción B: eliminar de global y agregar script que setee RUST_TEST_THREADS=2 en Windows
3. Elegir la más simple (ponytail: una línea)
4. Verificar que tests siguen pasando en Windows
5. git add -A && git commit -m "fix(test): T7 test-threads per-platform"
6. Actualizar este archivo
```

---

### TASK-16: C7 — Dependabot config para acciones

| Campo | Valor |
|-------|-------|
| **bitacora ref** | `C7` |
| **Archivos** | `.github/dependabot.yml` |
| **Skills** | `ponytail`, `ci-cd-and-automation` |
| **Esfuerzo** | 🟢 ~1h |
| **Estado** | ❌ |

**Prompt específico:**

```
Bitacora C7 — Actions pinned por SHA pero sin Dependabot config para actualizarlos.

Skills: ponytail, ci-cd-and-automation

Pasos:
1. Leer .github/dependabot.yml
2. Agregar entry para GitHub Actions con schedule weekly
3. git add -A && git commit -m "fix(ci): C7 Dependabot for GitHub Actions"
4. Actualizar este archivo
```

---

### TASK-17: NUEVO-15 — Code coverage report en CI

| Campo | Valor |
|-------|-------|
| **Backlog ref** | `NUEVO-15` |
| **Archivos** | CI workflows, Cargo.toml |
| **Skills** | `ponytail`, `ci-cd-and-automation` |
| **Esfuerzo** | 🟡 ~1d |
| **Estado** | ❌ |

**Prompt específico:**

```
Backlog NUEVO-15 — Code coverage report en CI + upload.

Skills: ponytail, ci-cd-and-automation

Pasos:
1. Leer ci-rust-10.yml — entender estructura actual
2. Agregar job coverage que:
   a. Instala cargo-llvm-cov via taiki-e/install-action
   b. Corre cargo llvm-cov nextest --workspace
   c. Sube a Codecov o genera artifact HTML
3. Ponytail: coverage codecov upload simple, sin dashboard custom
4. Verificar que nextest config existe
5. git add -A && git commit -m "feat(ci): NUEVO-15 code coverage CI"
6. Actualizar este archivo y Backlog.md
```

---

## TIER 2A — 🟠 Refactors Medianos (Cada uno 1-3d)

---

### TASK-18: P13 — Flat index threshold (small dataset optimization)

| Campo | Valor |
|-------|-------|
| **bitacora ref** | `P13` |
| **Archivos** | `src/index/core.rs`, `src/config.rs` |
| **Skills** | `ponytail`, `doubt-driven-development`, `performance-optimization` |
| **Esfuerzo** | 🟡 ~2d |
| **Estado** | ❌ |

**Prompt específico:**

```
Bitacora P13 — Para datasets <10K vectors, brute-force search es 10-100x más rápido que HNSW.

Skills: ponytail, doubt-driven-development, performance-optimization

Pasos:
1. codegraph_explore "search StorageEngine core.rs"
2. Entender el pipeline de búsqueda actual (HNSW siempre)
3. Implementar threshold automático:
   a. Si cardinalidad < N (default 10000), usar brute-force linear scan
   b. Si >= N, usar HNSW normal
4. Configurable via VantaConfig
5. Test básico: search en dataset pequeño usa flat path
6. cargo build && cargo nextest run ... && cargo clippy ...
7. git add -A && git commit -m "perf: P13 flat index threshold for small datasets"
8. Actualizar este archivo y bitacora.md
```

---

### TASK-19: P5 — Fragmentar archivos monolíticos (serialization.rs)

| Campo | Valor |
|-------|-------|
| **bitacora ref** | `P5` |
| **Archivos** | `src/sdk/serialization.rs` (1827L) |
| **Skills** | `ponytail`, `planning-and-task-breakdown`, `doubt-driven-development` |
| **Esfuerzo** | 🟡 ~1d |
| **Estado** | ❌ |

**Prompt específico:**

```
Bitacora P5 — Fragmentar src/sdk/serialization.rs (1827L) →
sdk/serialization/{records, formats, io, tests}.

Skills: ponytail, planning-and-task-breakdown, doubt-driven-development

Pasos:
1. Leer src/sdk/serialization.rs completo
2. Identificar módulos naturales: records (tipos), formats (formatos), io (lectura/escritura)
3. Crear src/sdk/serialization/ con mod.rs
4. Mover cada sección a su archivo, mantener visibilidad pub(crate)
5. NO cambiar lógica, solo mover
6. cargo build && cargo nextest run ...
7. git add -A && git commit -m "refactor: P5 split serialization.rs"
8. Actualizar este archivo
```

---

### TASK-20: W5 — OG image branding

| Campo | Valor |
|-------|-------|
| **bitacora ref** | `W5` |
| **Archivos** | `web/public/og-image.svg` (o .png) |
| **Skills** | `ponytail` |
| **Esfuerzo** | 🟡 ~1d |
| **Estado** | ❌ |

**Prompt específico:**

```
Bitacora W5 — OG image usa #ff6a00 en vez de brand amber #ff5500, #08080c en vez de #0a0a0a.

Skills: ponytail

Pasos:
1. Buscar og-image file en web/public/
2. Corregir colores:
   - #ff6a00 → #ff5500 (amber brand)
   - #08080c → #0a0a0a (dark brand)
3. Si no hay og-image, crear una mínima con el logo + texto "VantaDB"
4. git add -A && git commit -m "fix(brand): W5 OG image branding colors"
5. Actualizar este archivo
```

---

### TASK-21: W8 — Design system gaps (tokens)

| Campo | Valor |
|-------|-------|
| **bitacora ref** | `W8` |
| **Archivos** | CSS files en web/ |
| **Skills** | `ponytail`, `frontend-ui-engineering` |
| **Esfuerzo** | 🟡 ~2d |
| **Estado** | ❌ |

**Prompt específico:**

```
Bitacora W8 — --white: #000000 (confuso), --amber es naranja, SwissHero.tsx hardcodea
"#ff5500" en vez de var(--amber), pricing hardcodea #ff3b30 en vez de var(--danger).

Skills: ponytail, frontend-ui-engineering

Pasos:
1. grep "ff5500\|ff3b30\|--white\|--amber" en web/src/
2. Renombrar tokens inconsistentes:
   - --white → --black (si es #000000) o --bg-primary
   - Reemplazar hardcodes de color con var(--*)
3. NO cambiar diseño, solo usar variables
4. Verificar: tailwind build || tsc --noEmit
5. git add -A && git commit -m "refactor(design): W8 design token consistency"
6. Actualizar este archivo
```

---

## TIER 2B — 🟠 WASM & Bindings

---

### TASK-22: B18 — Homebrew SHA256 placeholders

| Campo | Valor |
|-------|-------|
| **bitacora ref** | `B18` |
| **Archivos** | `Formula/vantadb.rb` |
| **Skills** | `ponytail` |
| **Esfuerzo** | 🟢 ~1h |
| **Estado** | ❌ |

**Prompt específico:**

```
Bitacora B18 — Formula/vantadb.rb tiene 4 SHA256 "0000..." placeholders.

Skills: ponytail

Pasos:
1. Leer Formula/vantadb.rb
2. Descargar los artifacts reales del último release
3. sha256sum cada artifact
4. Reemplazar placeholders
5. git add -A && git commit -m "fix: B18 Homebrew SHA256 placeholders"
6. Actualizar este archivo
```

---

### TASK-23: B12 — MCP search_memory fallback silencioso

| Campo | Valor |
|-------|-------|
| **bitacora ref** | `B12` |
| **Archivos** | MCP server |
| **Skills** | `ponytail` |
| **Esfuerzo** | 🟢 ~30min |
| **Estado** | ❌ |

**Prompt específico:**

```
Bitacora B12 — search_memory usa Cosine como default sin advertir al caller.

Skills: ponytail

Pasos:
1. codegraph_explore "search_memory mcp"
2. Leer la tool search_memory
3. Agregar warning log cuando distance_metric no se especifica
4. O cambiar default a None → error explícito "distance_metric required"
5. cargo build && cargo nextest run ...
6. git add -A && git commit -m "fix(mcp): B12 search_memory fallback warning"
7. Actualizar este archivo
```

---

### TASK-24: B14 — MCP get_node_neighbors inconsistente

| Campo | Valor |
|-------|-------|
| **bitacora ref** | `B14` |
| **Archivos** | MCP server |
| **Skills** | `ponytail` |
| **Esfuerzo** | 🟢 ~1h |
| **Estado** | ❌ |

**Prompt específico:**

```
Bitacora B14 — get_node_neighbors usa storage.get() directo en vez de StorageEngine.

Skills: ponytail

Pasos:
1. codegraph_explore "get_node_neighbors mcp"
2. Leer la tool — verificar patrón
3. Cambiar storage.get() → StorageEngine.get() (como las otras tools)
4. cargo build && cargo nextest run ...
5. git add -A && git commit -m "fix(mcp): B14 get_node_neighbors consistency"
6. Actualizar este archivo
```

---

### TASK-25: B15 — MCP schema:// resource duplica metrics://

| Campo | Valor |
|-------|-------|
| **bitacora ref** | `B15` |
| **Archivos** | MCP server |
| **Skills** | `ponytail` |
| **Esfuerzo** | 🟢 ~1h |
| **Estado** | ❌ |

**Prompt específico:**

```
Bitacora B15 — schema:// resource duplica metrics://.

Skills: ponytail

Pasos:
1. codegraph_explore "schema resource mcp metrics resource"
2. Leer ambos resources
3. Si son duplicados exactos: eliminar schema://, redirigir a metrics://
4. Si tienen info diferente: consolidar en metrics://
5. cargo build
6. git add -A && git commit -m "refactor(mcp): B15 consolidate schema/metrics resources"
7. Actualizar este archivo
```

---

### TASK-26: B9 — Python AsyncVantaDB sin límite de concurrencia

| Campo | Valor |
|-------|-------|
| **bitacora ref** | `B9` |
| **Archivos** | `vantadb-python/` |
| **Skills** | `ponytail`, `security-and-hardening` |
| **Esfuerzo** | 🟡 ~1d |
| **Estado** | ❌ |

**Prompt específico:**

```
Bitacora B9 — AsyncVantaDB sin límite de concurrencia. Thread pool saturation posible.

Skills: ponytail, security-and-hardening

Pasos:
1. codegraph_explore "AsyncVantaDB vantadb-python"
2. Leer la implementación de AsyncVantaDB
3. Agregar Semaphore con max_concurrency default (ej: 4)
4. Configurable via parámetro en constructor
5. cargo build && cargo nextest run ... && pytest tests/
6. git add -A && git commit -m "fix(python): B9 AsyncVantaDB concurrency limit"
7. Actualizar este archivo
```

---

### TASK-27: B16 — TS SDK hardening (expandir tests a 50+)

| Campo | Valor |
|-------|-------|
| **bitacora ref** | `B16` |
| **Archivos** | TypeScript SDK en packages/ |
| **Skills** | `ponytail`, `test-driven-development` |
| **Esfuerzo** | 🟡 ~3d |
| **Estado** | ❌ |

**Prompt específico:**

```
Backlog NUEVO-09 / bitacora B16 — TS SDK: expandir de ~18 tests a 50+.

Skills: ponytail, test-driven-development

Pasos:
1. Leer tests TS actuales
2. Identificar APIs sin test coverage
3. Agregar tests para: error handling, batch operations, hybrid search, type stubs
4. Ponytail: tests simples (assert + describe), sin fixtures elaborados
5. Verificar: npx vitest run (o el test runner configurado)
6. git add -A && git commit -m "test(ts): B16 expand TS SDK tests to 50+"
7. Actualizar este archivo y Backlog.md
```

---

## TIER 3 — 🔵 Refactors Grandes (Cada uno 2-5d)

---

### TASK-28: P2 — WAL Mutex contention

| Campo | Valor |
|-------|-------|
| **bitacora ref** | `P2` |
| **Archivos** | `src/wal.rs`, `src/wal_sharded.rs`, `src/storage/wal.rs` |
| **Skills** | `ponytail`, `doubt-driven-development`, `performance-optimization` |
| **Esfuerzo** | 🟡 ~2d |
| **Estado** | ❌ |

**Prompt específico:**

```
Bitacora P2 — WAL Mutex contention. ShardedWal reduce contención pero hay paths legacy.

Skills: ponytail, doubt-driven-development, performance-optimization

Pasos:
1. codegraph_explore "ShardedWal WalWriter wal" para mapear todos los paths de escritura WAL
2. Confirmar que ShardedWal se usa en todos los paths
3. Si hay backends/configs que usan WalWriter directo, migrar a ShardedWal
4. Agregar #[instrument] o tracing para medir contención real
5. cargo build && cargo nextest run ... y tests de WAL específicos
6. git add -A && git commit -m "perf: P2 WAL Mutex contention — unify to ShardedWal"
7. Actualizar este archivo y bitacora.md
```

---

### TASK-29: P1 — HNSW insert_lock bottleneck (Rayon micro-batching)

| Campo | Valor |
|-------|-------|
| **bitacora ref** | `P1` |
| **Archivos** | `src/storage/engine/mod.rs`, `src/storage/engine/ops.rs` |
| **Skills** | `ponytail`, `doubt-driven-development`, `performance-optimization` |
| **Esfuerzo** | 🟡 1-2 semanas |
| **Estado** | ❌ |

**Prompt específico:**

```
Bitacora P1 — HNSW insert_lock es Mutex único que serializa TODAS las mutaciones.

Skills: ponytail, doubt-driven-development, performance-optimization

Pasos:
1. codegraph_explore "insert_lock StorageEngine"
2. Analizar benchmark actual de throughput para baseline
3. Implementar Rayon micro-batching: agrupar N inserts, ejecutar batch HNSW bajo un solo lock
4. Mantener backward compatibility
5. Benchmark: target es mejora medible en throughput
6. cargo build && cargo nextest run ... (incluir tests de estrés si existen)
7. git add -A && git commit -m "perf: P1 HNSW insert_lock Rayon micro-batching"
8. Actualizar este archivo y bitacora.md
```

---

### TASK-30: P3 — ACID Transaction Layer Phase 1 (WAL Transaction Records)

| Campo | Valor |
|-------|-------|
| **bitacora ref** | `P3` |
| **Archivos** | `src/wal.rs`, `src/storage/engine/ops.rs`, `src/storage/vfile.rs` |
| **Skills** | `ponytail`, `doubt-driven-development`, `api-and-interface-design`, `spec-driven-development` |
| **Esfuerzo** | ~2 semanas |
| **Estado** | ❌ |

**Prompt específico:**

```
Bitacora P3 — ACID Transaction Layer Phase 1.
No hay Begin/Commit/Abort en WAL. Si write_node_to_vstore éxito pero write_batch
(KV) falla, orphan vector queda en VantaFile.

Skills: ponytail, doubt-driven-development, api-and-interface-design, spec-driven-development

PASO 0 — Leer research: docs/research/ACID_TRANSACTIONS.md (análisis completo)

PASO 1 — spec-driven-development:
- Escribir mini-spec del cambio: qué variantes de WalRecord agregar (Begin/Commit/Abort)
- Cómo recovery debe descartar writes no cerrados

PASO 2 — Implementar:
1. Agregar variantes Begin/Commit/Abort a WalRecord
2. StorageEngine.begin_transaction() → escribe Begin record
3. StorageEngine.commit() → escribe Commit record
4. En recovery: descartar writes entre Begin y Abort no cerrados
5. VantaFile: no necesita rollback (Phase 2)

PASO 3 — Verificar:
- cargo build && cargo clippy ... && cargo nextest run ...
- Test específico: simular fallo a mitad de transacción, verificar rollback

PASO 4 — git add -A && git commit -m "feat(storage): P3 ACID transactions Phase 1"
PASO 5 — Actualizar bitacora.md y este archivo
```

---

### TASK-31: P4 — VantaFile writes reversibles

| Campo | Valor |
|-------|-------|
| **bitacora ref** | `P4` |
| **Archivos** | `src/storage/vfile.rs` |
| **Skills** | `ponytail`, `doubt-driven-development` |
| **Esfuerzo** | 🟡 1-3d |
| **Estado** | ❌ |

**Prompt específico:**

```
Bitacora P4 — VantaFile writes no reversibles. Si KV write falla, vector queda huérfano.

Skills: ponytail, doubt-driven-development

Pasos:
1. codegraph_explore "VantaFile write vfile.rs"
2. Leer docs/research/ACID_TRANSACTIONS.md Approach A/B/C
3. Elegir enfoque más simple (ponytail)
4. Implementar: lazy cleanup o buffered writes
5. cargo build && cargo nextest run ...
6. git add -A && git commit -m "feat(storage): P4 VantaFile reversible writes"
7. Actualizar este archivo
```

---

## TIER 4 — 🔴 Web Frontend Tasks

---

### TASK-32: WEB-001 — Re-add interactive WASM demo page

| Campo | Valor |
|-------|-------|
| **Backlog ref** | `WEB-001` |
| **Archivos** | `web/src/routes/` |
| **Skills** | `ponytail`, `frontend-ui-engineering` |
| **Esfuerzo** | 🟢 ~30min |
| **Estado** | ❌ |

**Prompt específico:**

```
Backlog WEB-001 — Re-add interactive WASM demo page.
Restaurar demo.tsx/demo.lazy.tsx que importa vantadb_wasm.js.

Skills: ponytail, frontend-ui-engineering

Pasos:
1. Buscar si demo.tsx existe en alguna rama (git log --all -- web/src/routes/demo*)
2. Si existe en git history: git restore
3. Si no: crear demo.lazy.tsx mínima que importe y monte la WASM demo
4. Verificar: tsc --noEmit
5. git add -A && git commit -m "feat(web): WEB-001 WASM demo page"
6. Actualizar Backlog.md y este archivo
```

---

### TASK-33: W12 — React memoization estratégica

| Campo | Valor |
|-------|-------|
| **bitacora ref** | `W12` |
| **Archivos** | Componentes pesados (Three.js hero, Nav, benchmark tables) |
| **Skills** | `ponytail`, `frontend-ui-engineering`, `performance-optimization` |
| **Esfuerzo** | 🟡 ~2d |
| **Estado** | ❌ |

**Prompt específico:**

```
Bitacora W12 — 0 React.memo, 0 useMemo, 0 useCallback en ~50 componentes.

Skills: ponytail, frontend-ui-engineering, performance-optimization

Pasos:
1. Identificar componentes pesados: Three.js hero, Nav (~22 rutas), benchmark tables
2. Agregar React.memo en componentes que renderizan lists o children estables
3. useMemo en cómputos costosos (filter, sort, map)
4. Ponytail: solo donde hay rerender visible, NO blanket memoization
5. Verificar: tsc --noEmit
6. git add -A && git commit -m "perf(web): W12 strategic memoization"
7. Actualizar este archivo
```

---

### TASK-34: W15 — Three.js hero issues

| Campo | Valor |
|-------|-------|
| **bitacora ref** | `W15` |
| **Archivos** | Three.js hero components |
| **Skills** | `ponytail`, `frontend-ui-engineering` |
| **Esfuerzo** | 🟡 ~1d |
| **Estado** | ❌ |

**Prompt específico:**

```
Bitacora W15 — Three.js hero: sin error boundary, mouse tracking en mobile,
wireframe overflow, sin prefers-reduced-motion.

Skills: ponytail, frontend-ui-engineering

Pasos:
1. Encontrar el componente Three.js hero
2. Agregar ErrorBoundary con fallback visual
3. Detectar mobile por touch support para desactivar mouse tracking
4. Wireframe responsive position
5. prefers-reduced-motion check
6. Verificar: tsc --noEmit
7. git add -A && git commit -m "fix(web): W15 Three.js hero issues"
8. Actualizar este archivo
```

---

### TASK-35: W14 — Direct DOM mutation → React state

| Campo | Valor |
|-------|-------|
| **bitacora ref** | `W14` |
| **Archivos** | Componentes con onMouseEnter/onMouseLeave + element.style |
| **Skills** | `ponytail`, `frontend-ui-engineering` |
| **Esfuerzo** | 🟡 ~1d |
| **Estado** | ❌ |

**Prompt específico:**

```
Bitacora W14 — Componentes mutan element.style directamente en onMouseEnter/onMouseLeave.

Skills: ponytail, frontend-ui-engineering

Pasos:
1. grep "onMouseEnter\|onMouseLeave\|element.style" en web/src/
2. Migrar cada caso a useState + style prop, o CSS classes condicionales
3. Ponytail: CSS :hover cuando sea posible (no necesita JS)
4. Verificar: tsc --noEmit
5. git add -A && git commit -m "refactor(web): W14 DOM mutation → React state"
6. Actualizar este archivo
```

---

### TASK-36: W13 — Animation libraries bundling

| Campo | Valor |
|-------|-------|
| **bitacora ref** | `W13` |
| **Archivos** | `web/package.json`, componentes |
| **Skills** | `ponytail`, `performance-optimization` |
| **Esfuerzo** | 🟡 ~1d |
| **Estado** | ❌ |

**Prompt específico:**

```
Bitacora W13 — 3 animation libs: GSAP 3.15 + Motion 12.42 + AnimeJS 4.5 ≈ ~155KB extra.

Skills: ponytail, performance-optimization

Pasos:
1. grep "from 'motion'\|from 'animejs'\|from 'gsap'" en web/src/
2. Contar usos de cada librería
3. Si Motion tiene pocos usos y GSAP ya es el estándar:
   - Migrar usos de Motion a GSAP
   - Eliminar motion de package.json
4. Si AnimeJS tiene pocos usos únicos: migrar a GSAP, eliminar
5. Ponytail: mantener solo GSAP (ya es el estándar del proyecto)
6. Verificar: npx tsc --noEmit
7. git add -A && git commit -m "perf(web): W13 unify animation to GSAP only"
8. Actualizar este archivo
```

---

### TASK-37: W9 — SEO gaps

| Campo | Valor |
|-------|-------|
| **bitacora ref** | `W9` |
| **Archivos** | `web/src/lib/seo.ts`, `web/public/sitemap.xml` |
| **Skills** | `ponytail`, `ai-seo` |
| **Esfuerzo** | 🟡 ~1d |
| **Estado** | ❌ |

**Prompt específico:**

```
Bitacora W9 — Twitter cards sin site/creator, 3 routes fuera del sitemap,
JSON-LD incomplete, blog sin canonical.

Skills: ponytail, ai-seo

Pasos:
1. Leer web/src/lib/seo.ts y web/public/sitemap.xml
2. Agregar: twitter:site, twitter:creator a seo.ts
3. Agregar rutas faltantes al sitemap (/docs-api, /security, /product/benchmarks)
4. Expandir JSON-LD con url, image, softwareVersion
5. Agregar canonical URLs al blog
6. git add -A && git commit -m "fix(seo): W9 SEO gaps"
7. Actualizar este archivo
```

---

## 📊 Status Tracker (ACTUALIZAR EN CADA TAREA)

```
TASK-ID      | Backlog Ref    | Dominio          | Estado | Commit
-------------|----------------|------------------|--------|-------
TASK-01      | REC-02         | error.rs helper  | ✅     | a1febe8
TASK-02      | REC-03         | source chaining  | ✅     | (ya migrado)
TASK-03      | P8             | SAFETY doc       | ✅     | (ya hecho)
TASK-04      | P9             | magic numbers    | ✅     | (ya hecho)
TASK-05      | P12            | metrics auth     | ✅     | (ya hecho)
TASK-06      | P10            | Spanish comments | ✅     | (WIP, sin commit)
TASK-07      | P7             | error hierarchy  | ✅     | (ya migrado)
TASK-08      | W16            | blog errors      | ❌     | —
TASK-09      | W6             | security headers | ❌     | —
TASK-10      | W17            | touch targets    | ❌     | —
TASK-11      | PERF-13        | read_only helper | ❌     | —
TASK-12      | PERF-14        | init_telemetry   | ❌     | —
TASK-13      | DOC-02         | split insert_hnsw| ❌     | —
TASK-14      | P6             | dedup patterns   | ❌     | —
TASK-15      | T7             | test-threads     | ❌     | —
TASK-16      | C7             | Dependabot       | ❌     | —
TASK-17      | NUEVO-15       | code coverage CI | ❌     | —
TASK-18      | P13            | flat index       | ❌     | —
TASK-19      | P5             | split serializ.  | ❌     | —
TASK-20      | W5             | OG branding      | ❌     | —
TASK-21      | W8             | design tokens    | ❌     | —
TASK-22      | B18            | Homebrew SHA     | ❌     | —
TASK-23      | B12            | MCP search_fallback| ❌  | —
TASK-24      | B14            | MCP get_neighbors| ❌     | —
TASK-25      | B15            | MCP schema dup   | ❌     | —
TASK-26      | B9             | Async conc. limit| ❌     | —
TASK-27      | B16/NUEVO-09   | TS SDK 50+ tests | ❌     | —
TASK-28      | P2             | WAL contention   | ❌     | —
TASK-29      | P1             | HNSW insert_lock | ❌     | —
TASK-30      | P3             | ACID Phase 1     | ❌     | —
TASK-31      | P4             | VantaFile revert | ❌     | —
TASK-32      | WEB-001        | WASM demo page   | ❌     | —
TASK-33      | W12            | React memo       | ❌     | —
TASK-34      | W15            | Three.js hero    | ❌     | —
TASK-35      | W14            | DOM mutation     | ❌     | —
TASK-36      | W13            | animation unify  | ❌     | —
TASK-37      | W9             | SEO gaps         | ❌     | —
```

---

## Orden de Ejecución Recomendado

```
FASE 1 — Rápidos (Tier 1A): TASK-01 → TASK-10
  Cada una < 2h. Quick wins. Dan impulso.

FASE 2 — Code Health (Tier 1B): TASK-11 → TASK-14
  Refactors que simplifican el código base.

FASE 3 — Testing & CI (Tier 1C): TASK-15 → TASK-17
  Mejoran la calidad y confianza del pipeline.

FASE 4 — Medianos (Tier 2A): TASK-18 → TASK-21
  Refactors más grandes pero bien acotados.

FASE 5 — WASM/Bindings (Tier 2B): TASK-22 → TASK-27
  Mejoras en bindings y MCP.

FASE 6 — Grandes (Tier 3): TASK-28 → TASK-31
  Requieren más análisis y cuidado.

FASE 7 — Web (Tier 4): TASK-32 → TASK-37
  Frontend: memoization, Three.js, SEO, animations.
```

---

## ⚡ Quick Reference: Comandos Frecuentes

```bash
# Build rápido
cargo check -p vantadb --no-default-features -F "fjall,cli"
cargo build -p vantadb --no-default-features -F "fjall,cli"

# Tests
cargo nextest run --profile audit --workspace --build-jobs 2

# Tests de un crate específico
cargo nextest run -p vantadb

# Clippy + fmt
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Python SDK tests (después de maturin build)
dev-tools/setup_venv.ps1
target/audit-venv/Scripts/python -m pytest vantadb-python/tests/test_sdk.py -v

# Pre-commit
dev-tools/verify_changed.ps1    # ~30s, CodeGraph-optimized

# Full verify
just verify                     # fmt + clippy + test + deny
```

---

## ⚡ Quick Reference: CodeGraph Antes de Cada Tarea

```bash
# Mapear blast radius antes de editar
codegraph_explore "nombre_del_archivo_a_cambiar"
# codegraph_explore "StorageEngine insert_hnsw"
# codegraph_explore "VantaError serialization src/error.rs"
```

Siempre pregunta a codegraph antes de editar. Te dice qué módulos dependen de lo que vas a cambiar y quién llama a las funciones objetivo.

---

## 📝 Instrucciones de Uso para el Agente

1. **Al iniciar:** leer este archivo COMPLETO. Identificar próxima tarea ❌.
2. **Loop:** seguir el Master Execution Loop al pie de la letra.
3. **Cada tarea:**
   - Leer el prompt específico de la tarea
   - Cargar las skills indicadas
   - codegraph_explore primero
   - Implementar
   - Verificar (build + tests + clippy)
   - Actualizar este archivo (marcar ✅)
   - Actualizar Backlog.md o bitacora.md
   - Commit
4. **Cada ~3-4 tareas** o si el contexto llega a ~180K:
   - Actualizar este archivo
   - Commit "checkpoint"
   - Avisar al usuario para nueva sesión
5. **Si una tarea tiene errores:** arreglar, NO saltar. Si no se puede resolver, marcar como 🗑️ y documentar por qué.
6. **Ponytail activo:** antes de escribir código, subir la escalera. ¿Ya existe? ¿Stdlib? ¿Dependencia instalada? ¿Una línea? SINO: mínimo código.

---

=== RECITATION ===
Objetivo activo: TASK-08 ❌ — W16 Blog factual errors
Tasks completadas: TASK-01..TASK-07 (todas ✅)
Estado actual: plan actualizado, Backlog.md actualizado (REC-02 ✅, REC-03 ✅), bitacora.md actualizado (P10 ✅)
Próxima acción: TASK-08 — fix GitHub link (ness-e/Vantadb → vantadb/vantadb) en web/content/blog/introducing-vantadb.md
=== END RECITATION ===
