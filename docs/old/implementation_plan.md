# Operación Rescate: Estabilización del Core

## Diagnóstico Confirmado

Tras investigación profunda del repositorio, confirmo el diagnóstico sistémico:

| Problema | Evidencia Concreta |
|----------|-------------------|
| **Naming dual** | `Cargo.toml` → `connectomedb`, README → `NexusDB`, Dockerfile WORKDIR → `/usr/src/nexusdb`, docker-compose → `connectomedb-server`, `start.sh` → "NexusDB Intelligent Entrypoint", env vars → `CONNECTOMEDB_*` |
| **HNSW truncado** | `index.rs:131` — MVP comment: "Just fully connect to entry point across valid layers". No `M` limit, no `ef_construction`, no proper neighbor selection |
| **Docs infladas** | `docs/complete/` tiene 43 archivos incluyendo "Bayesian Forgetfulness", "Synaptic Depression", "Contextual Priming", "Logical Immunology". `docs/business/` con investor pitch, marketing, monetization |
| **Build inestable** | Dockerfile raíz usa `rust:slim-bookworm` (sin version pin), `docker/Dockerfile` usa `rust:1.82-slim-bullseye`. No hay `rust-toolchain.toml` |
| **Duplicados** | 2 Dockerfiles de producción, 2 start.sh, `tests_graph_db/` y `tests_server_db/` en .gitignore pero existentes, `todo.md` y `todo.txt` idénticos (~2.3MB cada uno) |
| **Metáforas biológicas en código** | `Neuron = UnifiedNode`, `Synapse = Edge`, `NeuronType::STNeuron/LTNeuron`, `semantic_valence`, `CognitiveUnit` trait |

---

## User Review Required

> [!IMPORTANT]
> **Decisión de nombre: basándome en la investigación de `InvestigacionNombresPosibles.md`**, los 3 candidatos con menor riesgo son:
>
> | Nombre | Dominio .com | GitHub | Riesgo | Por qué |
> |--------|-------------|--------|--------|---------|
> | **VantaDB** | ✅ Libre | ✅ Libre | Bajo | Cero conflictos encontrados. Suena premium, moderno, tecnológico |
> | **KairoDB** | ✅ Libre | ✅ Libre | Bajo | Sin conflictos (KairosDB es TimeSeries, pero "Kairo" ≠ "Kairos") |
> | **ZynkDB** | ✅ Libre | ✅ Libre | Bajo | Único, memorable, sin conflictos |
>
> **Mi recomendación: VantaDB** — suena profesional, no tiene metáfora biológica, dominio limpio, ningún conflicto. "Vanta" evoca profundidad (Vantablack) sin prometer nada irrealizable.
>
> **¿Confirmas VantaDB o prefieres otro nombre?** Todo el plan asume `{NOMBRE}` como placeholder hasta tu decisión.

> [!WARNING]
> **Archivos que serán ELIMINADOS** (confirmar):
> - `docs/business/` completo (investor pitch, marketing, monetization, GTM timeline)
> - `docs/complete/` — specs aspiracionales (20+ docs de "Bayesian Forgetfulness", "Synaptic Depression", etc.)
> - `docDev/article_dev_to_draft.md`
> - `todo.md` + `todo.txt` (~4.6MB de peso muerto)
> - `strategic_master_plan.md.resolved`
> - `deep-research-report.md` + `InvestigacionNombresPosibles.md` (ya procesados, se archivan en `/docs/archive/`)
> - `build_bench.log`, `md_list.txt`, `collect_code.ps1`
> - `Dockerfile.bench` (se reescribe en `/docker/`)
> - `docker/Dockerfile` duplicado (se unifica)
> - `.connectome_profile` (raíz + nexusdb-python)
> - `BENCHMARKS.md`, `CHANGELOG.md` actuales (se reescriben honestos)
> - `CONTRIBUTING.md`, `SECURITY.md` (se reescriben alineados al nombre)
> - `high_density_bench_db/`, `tests_graph_db/`, `tests_server_db/` (artifacts de test, ya en .gitignore)

> [!CAUTION]
> **Las metáforas biológicas serán eliminadas del código público:**
> - `Neuron` type alias → eliminado
> - `Synapse` type alias → eliminado
> - `NeuronType::STNeuron/LTNeuron` → `NodeTier::Hot/Cold`
> - `CognitiveUnit` trait → `AccessTracker` trait
> - `semantic_valence` field → `importance` field
> - `neuron_type` field → `tier` field
> - Magic header `CXHNSW01` → `{NOMBRE}01` (ej: `VNTAHNSW`)
> - Todos los comments con "neuronal", "brain", "cortex", "amygdala" → lenguaje técnico neutro

---

## Proposed Changes

### FASE 0 — Congelación + Nombre (Día 1)

---

#### [NEW] [rust-toolchain.toml](file:///c:/PROYECTOS/connectomadb/ConnectomeDB/rust-toolchain.toml)
Pin del toolchain a stable, garantiza builds reproducibles:
```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
```

#### [MODIFY] [Cargo.toml](file:///c:/PROYECTOS/connectomadb/ConnectomeDB/Cargo.toml)
- `name` → `"{nombre}"` (ej: `"vantadb"`)
- `description` → descripción honesta sin metáforas
- `keywords` → `["database", "vector", "graph", "embedded", "rust"]`
- Eliminar workspace member `nexusdb-python` (se mueve fuera del workspace por ahora)
- Binario: `connectome-server` → `{nombre}-server`
- Binario CLI: `connectome-cli` → `{nombre}-cli`
- Tests: mantener todos los existentes pero renombrar paths si cambian

#### [MODIFY] [Cargo.lock](file:///c:/PROYECTOS/connectomadb/ConnectomeDB/Cargo.lock)
Se regenera automáticamente tras cambiar Cargo.toml.

---

### FASE 1 — Unificación Estructural (Días 2-4)

---

#### Renaming Global (todos los archivos fuente)

Archivos afectados en `src/`:
- [lib.rs](file:///c:/PROYECTOS/connectomadb/ConnectomeDB/src/lib.rs) — Reescribir doc comment, eliminar aliases biológicos, eliminar re-exports `Neuron`/`Synapse`
- [node.rs](file:///c:/PROYECTOS/connectomadb/ConnectomeDB/src/node.rs) — Eliminar `type Neuron`, `type Synapse`, renombrar `NeuronType` → `NodeTier`, `CognitiveUnit` → `AccessTracker`, `neuron_type` → `tier`, `semantic_valence` → `importance`
- [error.rs](file:///c:/PROYECTOS/connectomadb/ConnectomeDB/src/error.rs) — `ConnectomeError` → `{Nombre}Error` (ej: `VantaError`)
- [engine.rs](file:///c:/PROYECTOS/connectomadb/ConnectomeDB/src/engine.rs) — Actualizar imports
- [index.rs](file:///c:/PROYECTOS/connectomadb/ConnectomeDB/src/index.rs) — Magic header `CXHNSW01` → nuevo header
- [bin/connectome-server.rs](file:///c:/PROYECTOS/connectomadb/ConnectomeDB/src/bin/connectome-server.rs) → renombrar archivo a `{nombre}-server.rs`
- [bin/connectome-cli.rs](file:///c:/PROYECTOS/connectomadb/ConnectomeDB/src/bin/connectome-cli.rs) → renombrar archivo a `{nombre}-cli.rs`
- Todos los demás `.rs` en src/ — actualizar `use connectomedb::` → `use {nombre}::`

Tests afectados:
- Todos los 27 archivos en `tests/` — actualizar `use connectomedb::` → `use {nombre}::`

Benchmarks:
- 3 archivos en `benches/` — actualizar imports

#### Estructura de Documentación

```
docs/
├── architecture.md    ← Reescrito: diseño real, sin metáforas
├── decisions.md       ← NUEVO: registro de decisiones técnicas
├── roadmap.md         ← Reescrito: ejecución real, no aspiracional
├── api/
│   └── IQL.md         ← Se conserva (gramática de queries)
├── operations/
│   └── CONFIGURATION.md ← Se conserva (config real)
└── archive/           ← Investigación ya procesada
    ├── deep-research-report.md
    └── name-research.md
```

#### [DELETE] Archivos eliminados
- `docs/business/` (9 archivos + 1 directorio)
- `docs/complete/` (43 archivos + 20 directorios)
- `docs/prompts/`, `docs/research/`, `docs/info/`, `docs/assets/`
- `docDev/` completo
- `todo.md`, `todo.txt`
- `strategic_master_plan.md.resolved`
- `build_bench.log`, `md_list.txt`, `collect_code.ps1`
- `.connectome_profile` (raíz)

#### Docker — Unificación

#### [MODIFY] [Dockerfile](file:///c:/PROYECTOS/connectomadb/ConnectomeDB/Dockerfile)
- Pin Rust version: `FROM rust:1.82-slim-bookworm`
- `WORKDIR /usr/src/{nombre}`
- Build binary: `{nombre}-server`
- Eliminar rutas inconsistentes

#### [DELETE] [docker/Dockerfile](file:///c:/PROYECTOS/connectomadb/ConnectomeDB/docker/Dockerfile)
Duplicado. Se unifica en el Dockerfile raíz.

#### [DELETE] [docker/start.sh](file:///c:/PROYECTOS/connectomadb/ConnectomeDB/docker/start.sh)
Duplicado.

#### [MODIFY] [docker-compose.yml](file:///c:/PROYECTOS/connectomadb/ConnectomeDB/docker-compose.yml)
- Service: `{nombre}` (no `connectomedb`)
- Container: `{nombre}-server`
- Env vars: `{NOMBRE}_*` (ej: `VANTADB_PORT`, `VANTADB_MEMORY_LIMIT`)
- Volumes: `{nombre}_data`

#### [MODIFY] [start.sh](file:///c:/PROYECTOS/connectomadb/ConnectomeDB/start.sh)
- Comments: `{Nombre} Entrypoint`
- Env: `{NOMBRE}_MEMORY_LIMIT`
- Exec: `{nombre}-server`

#### [MODIFY] [.gitignore](file:///c:/PROYECTOS/connectomadb/ConnectomeDB/.gitignore)
Limpiar referencias a `connectome`, `nexusdb`. Añadir reglas correctas.

---

### FASE 2 — HNSW Estabilización (Días 5-10)

---

#### [MODIFY] [index.rs](file:///c:/PROYECTOS/connectomadb/ConnectomeDB/src/index.rs)
Reescritura del HNSW con algoritmo correcto:

**Cambios clave:**
1. **Parámetros configurables:**
   ```rust
   pub struct HnswConfig {
       pub m: usize,              // Max connections per layer (default: 16)
       pub m_max0: usize,         // Max connections at layer 0 (default: 2*M = 32)
       pub ef_construction: usize, // Build-time beam width (default: 200)
       pub ef_search: usize,      // Search-time beam width (default: 50)
       pub ml: f64,               // Level generation factor (default: 1/ln(M))
   }
   ```

2. **Inserción correcta por capas:**
   - Seleccionar capa aleatoria con `-ln(uniform) * ml`
   - Para cada capa de `max_layer` hasta `node_layer + 1`: greedy search hacia el más cercano
   - Para cada capa de `min(node_layer, max_layer)` hasta 0: encontrar `ef_construction` vecinos más cercanos, seleccionar `M` (o `M_max0` en capa 0) como conexiones

3. **Búsqueda correcta con ef_search:**
   - Descender desde entry point por capas superiores (greedy, ef=1)
   - En capa 0: beam search con `ef_search` candidatos
   - Retornar top-k del resultado

4. **Neighbor selection heuristic:**
   - Implementar selección simple (keep closest M) como v1
   - Preparar para heuristic selection (Malkov's algorithm 4) como v2

#### [NEW] [tests/hnsw_recall.rs](file:///c:/PROYECTOS/connectomadb/ConnectomeDB/tests/hnsw_recall.rs)
Test de recall obligatorio:
- Insertar 10,000 vectores random de 128 dimensiones
- Comparar HNSW vs brute-force
- Medir `recall@10` (debe ser ≥ 0.95 con M=16, ef_construction=200, ef_search=100)
- Medir latencia de inserción y búsqueda
- Medir uso de memoria

---

### FASE 3 — Gobernanza y Documentación (Días 11-13)

---

#### [NEW] [AGENT.md](file:///c:/PROYECTOS/connectomadb/ConnectomeDB/AGENT.md)
Reglas de ingeniería del proyecto (como las definiste en tu plan).

#### [NEW] [CLAUDE.md](file:///c:/PROYECTOS/connectomadb/ConnectomeDB/CLAUDE.md)
Contexto para asistentes AI (como lo definiste).

#### [MODIFY] [README.MD](file:///c:/PROYECTOS/connectomadb/ConnectomeDB/README.MD)
Reescritura completa:
1. Qué es `{Nombre}` (1 párrafo, sin metáforas)
2. Qué hace HOY (lista honesta de capacidades reales)
3. Qué NO hace (declaración explícita de limitaciones)
4. Quickstart (cómo compilar y correr)
5. Benchmarks reales (con datos de `tests/hnsw_recall.rs`)
6. Estado del proyecto (versión `0.1.0`, pre-alpha)

#### [NEW] [docs/architecture.md](file:///c:/PROYECTOS/connectomadb/ConnectomeDB/docs/architecture.md)
Diseño real: `UnifiedNode`, `CPIndex`, `InMemoryEngine`, `WalWriter`. Sin metáforas.

#### [NEW] [docs/decisions.md](file:///c:/PROYECTOS/connectomadb/ConnectomeDB/docs/decisions.md)
Registro de decisiones arquitectónicas (ADRs). Primera entrada: "Elegir HNSW propio vs wrapper".

#### [NEW] [docs/roadmap.md](file:///c:/PROYECTOS/connectomadb/ConnectomeDB/docs/roadmap.md)
Roadmap real con SemVer:
- `0.1.0`: Core compila, búsqueda brute-force, HNSW básico
- `0.2.0`: HNSW funcional con recall ≥ 0.95
- `0.3.0`: Persistencia RocksDB estable
- `0.4.0`: API HTTP mínima
- `1.0.0`: MVP estable (benchmark, API, persistencia, tests)

---

### FASE 4 — CI/CD (Día 14)

---

#### [MODIFY] [.github/workflows/rust_ci.yml](file:///c:/PROYECTOS/connectomadb/ConnectomeDB/.github/workflows/rust_ci.yml)
- Renombrar workflow: `{Nombre} CI`
- Añadir steps:
  - `cargo fmt --check`
  - `cargo clippy -- -D warnings`
  - `cargo build --locked`
  - `cargo test -- --test-threads=2`
  - `cargo bench --no-run`

#### [MODIFY] [.github/workflows/release.yml](file:///c:/PROYECTOS/connectomadb/ConnectomeDB/.github/workflows/release.yml)
- Actualizar binario name a `{nombre}-server`
- Actualizar referencias

---

## Open Questions

> [!IMPORTANT]
> 1. **¿Nombre final?** VantaDB, KairoDB, ZynkDB, o ¿otro?
> 2. **¿Renombrar el directorio raíz del repositorio?** Actualmente es `ConnectomeDB`. ¿Lo renombramos a `{nombre}` también o solo internamente?
> 3. **¿Renombrar el repositorio de GitHub?** De `ness-e/ConnectomeDB` a `ness-e/{nombre}`?
> 4. **¿El workspace member `nexusdb-python`?** ¿Lo eliminamos del workspace por ahora o lo renombramos a `{nombre}-python` y lo mantenemos?
> 5. **¿Eliminar la dependencia de RocksDB por ahora?** El core usa `InMemoryEngine` (HashMap). RocksDB solo se usa en `storage.rs` para persistencia. Podría simplificar enormemente el build (RocksDB es la dependencia más pesada y problemática).

---

## Verification Plan

### Automated Tests
```bash
# FASE 1: Build reproduce
cargo build --locked
cargo test -- --test-threads=2

# FASE 2: HNSW recall
cargo test --test hnsw_recall -- --nocapture

# FASE 3: Code quality
cargo fmt --check
cargo clippy -- -D warnings

# FASE 4: Benchmarks
cargo bench --bench high_density --no-run
```

### Manual Verification
- Grep global por `ConnectomeDB`, `NexusDB`, `connectomedb`, `nexusdb`, `Neuron`, `Synapse`, `neuron` → debe dar 0 resultados (excepto en `docs/archive/`)
- Verificar que README refleja estado real
- Verificar que CI pasa en GitHub Actions
- Revisar que no hay archivos huérfanos o duplicados
