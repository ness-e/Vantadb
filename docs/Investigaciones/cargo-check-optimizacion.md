# Investigación: Optimización de `cargo check`

> **Propósito:** Acelerar `cargo check` en desarrollo local sin romper el proyecto.
> **Restricción conocida:** `jobs = 2` en `.cargo/config.toml` obligatorio por RAM (OS error 1455 en Windows con page file limitado).
> **Contexto:** 548 dependencias, 16 crates en workspace, 9 librerías nativas C/C++, 23 proc-macros.
> **Última revisión:** 2026-07-13

---

## Índice de Tareas (por orden recomendado)

| # | Tarea | Riesgo | Impacto en speed | Esfuerzo |
|---|-------|--------|-----------------|----------|
| 1 | Profile `check` personalizado | ❌ No aplicable (reservado) | — | — |
| 2 | Quitar `target-cpu=native` | Bajo | Medio | 2 min |
| 3 | Podar deps no usadas (`cargo machete`) | Bajo | Bajo | 15 min |
| 4 | sccache local | Bajo | Alto (rebuilds) | 30 min |
| 5 | Mover features heavies fuera de default | Medio | Alto | 1-2 hr |
| 6 | Separar adapters en otro workspace | Medio-Alto | Muy Alto | 2-4 hr |
| 7 | Consolidar deps duplicadas | Bajo-Medio | Medio | 1-3 hr |

---

## Tarea 1 — Profile `check` personalizado

### Objetivo

Crear un perfil de compilación más ligero que `dev` (opt-level=1) para `cargo check`.

### Diagnóstico

El profile `dev` actual usa `opt-level=1`, lo que fuerza a LLVM a aplicar optimizaciones ligeras durante codegen. Para `cargo check` (que solo verifica tipos, no genera código ejecutable), `opt-level=0` es suficiente y más rápido.

**Sin embargo:** El nombre `check` es un nombre reservado por Cargo ([docs](https://doc.rust-lang.org/cargo/reference/profiles.html)). No se puede crear `[profile.check]`.

**El profile `check` nativo ya es óptimo:**
- `opt-level = 0` (por defecto)
- `debug = 0` (por defecto)
- `codegen-units = 256` (por defecto, porque no hay LTO en check)

Es decir, `cargo check` ya usa estas settings. No hay nada que tunear.

### Veredicto

**❌ No aplicable.** El profile check built-in de Cargo ya es óptimo. No hay ganancia posible.

---

## Tarea 2 — Quitar `target-cpu=native`

### Objetivo

Eliminar `-C target-cpu=native` de `rustflags` en `.cargo/config.toml` para reducir tiempo de codegen de LLVM.

### Diagnóstico

`target-cpu=native` le dice a LLVM: "optimiza para la CPU exacta de esta máquina". Esto:
- Activa pases de optimización específicos de microarquitectura (ej: AVX-512, BMI2, etc.)
- Aumenta el tiempo de codegen porque LLVM tiene que analizar la CPU host y aplicar optimizaciones específicas
- Genera binarios que NO corren en CPUs más viejas

Sin esa flag, LLVM usa `x86-64-v2` como baseline (el default de `x86_64-pc-windows-msvc`), que corre en cualquier CPU post-2009.

### Plan de acción

1. En `.cargo/config.toml`, cambiar:

```toml
# Antes:
rustflags = ["-C", "target-cpu=native"]

# Después:
# rustflags = ["-C", "target-cpu=native"]  # comentado
# O directamente:
rustflags = []
```

2. O simplemente borrar la línea `rustflags`

### Blast radius

| Ítem | Impacto |
|------|---------|
| **Rendimiento runtime** | Marginal (<2% en benchmarks sintéticos). El 99% del código de VantaDB no es hot-path de CPU |
| **HNSW/vector search** | Si hay hot paths que se beneficien de AVX-512, se pueden marcar con `#[target_feature(enable = "avx512f")]` específico |
| **CI** | CI no usa `target-cpu=native`, 0 impacto |
| **Compatibilidad** | El binario compilado local ahora corre en cualquier CPU x86-64. Beneficio, no riesgo |

### Post-tarea

- [ ] Verificar: `cargo check -p vantadb` compila (no hay flags inválidas)
- [ ] Verificar: `cargo nextest run --profile audit -p vantadb -- test_core_invariants` pasa
- [ ] Medir: diferencia en tiempo de `cargo check -p vantadb` con/sin la flag

---

## Tarea 3 — Podar deps no usadas

### Objetivo

Eliminar dependencias declaradas en Cargo.toml que no se usan en código.

### Diagnóstico

`cargo-machete` ya está instalado y configurado en el proyecto. Escanea el árbol de imports y detecta dependencias declaradas pero nunca usadas.

Actualmente hay `ignored` configurados en:
- `vantadb-enterprise/Cargo.toml`: ignora `vantadb`
- `vantadb-wasm/Cargo.toml`: ignora `getrandom`

Ignorar es necesario cuando una dependencia es usada indirectamente (ej: `vantadb` se usa como path dep pero machete no detecta uso directo de símbolos).

### Plan de acción

1. Ejecutar: `cargo machete`
2. Revisar cada falso positivo
3. Agregar `ignored` para los falsos positivos
4. Eliminar las dependencias realmente no usadas

### Blast radius

| Ítem | Impacto |
|------|---------|
| **Compilación** | Menos código a bajar y compilar |
| **Cargo.lock** | Se simplifica si se eliminan dependencias |
| **Riesgo** | Bajo — machete es conservador. Falsos positivos con proc-macros (ej: `serde` con "derive" feature) |

### Comandos útiles

```bash
# Ejecutar machete
cargo machete

# Para ver qué depende de qué (confirmar no-uso)
cargo tree -p vantadb

# Para detectar features no usadas dentro de dependencias
cargo +nightly info --unused-features   # experimental
```

### Post-tarea

- [ ] `cargo machete` reporta 0 findings (o ignorados justificados)
- [ ] `cargo check --workspace` compila
- [ ] `cargo nextest run --profile audit --workspace` pasa

---

## Tarea 4 — sccache local

### Objetivo

Cachear objetos compilados entre builds para acelerar rebuilds (~40%).

### Diagnóstico

`sccache` (Mozilla) es un cache de compilación estilo ccache pero para Rust. Almacena objetos `.o` compilados y los reusa si el código fuente no cambió.

CI ya usa `mozilla-actions/sccache-action`, pero localmente no está configurado.

### Plan de acción

1. Instalar: `cargo install sccache`
2. Iniciar servidor: `sccache --start-server`
3. Configurar en `.cargo/config.toml`:

```toml
[build]
jobs = 2
# Eliminar o comentar rustc-wrapper si usabas otra herramienta
# rustc-wrapper = "sccache"
```

O via variable de entorno (recomendado para no contaminar CI):

```bash
# PowerShell (perfil de usuario)
$env:RUSTC_WRAPPER = "sccache"
# O en ~/.cargo/config.toml
```

4. Verificar: `sccache --show-stats`

### Blast radius

| Ítem | Impacto |
|------|---------|
| **Windows** | sccache funciona en Windows. Requiere `sccache --start-server` manual o servicio |
| **CI** | CI usa su propio sccache action. No se afecta |
| **Rebuilds** | Primer build: igual o más lento (poblado de cache). Builds subsiguientes: ~40% más rápido |
| **Compatibilidad** | sccache es compatible con link.exe, cl.exe, y `jobs = 2` |

### Consideraciones Windows

- sccache en Windows a veces tiene problemas con rutas largas (`MAX_PATH`)
- El servidor sccache puede consumir ~200MB de RAM
- Si hay crashes de sccache, se puede desactivar con `$env:RUSTC_WRAPPER=""`

### Post-tarea

- [ ] `sccache --show-stats` muestra hits en rebuilds
- [ ] `cargo check -p vantadb` funciona correctamente con RUSTC_WRAPPER activo
- [ ] Primer build: poblado de cache. Segundo build: hits

---

## Tarea 5 — Mover features heavies fuera de default

### Objetivo

Sacar `rocksdb`, `arrow`, `advanced-tokenizer` (tantivy) y `prometheus` de las features default para que `cargo check` no compile esas dependencias pesadas a menos que se pidan explícitamente.

### Diagnóstico

Default features actuales en `Cargo.toml:86`:

```
default = ["cli", "arrow", "rocksdb", "fjall", "sysinfo", "memmap2", "fs2", "prometheus", "rayon", "advanced-tokenizer"]
```

Heavies:
| Feature | Dependencia | Peso |
|---------|-------------|------|
| `rocksdb` | `rocksdb 0.24.0` + `librocksdb-sys` | Compila ~10s de C++ en MSVC |
| `arrow` | `arrow 59` + ~15 sub-crates | GIGANTE. Casi todo el ecosistema Arrow |
| `advanced-tokenizer` | `tantivy 0.26` + stopwords | Full-text search engine completo |
| `prometheus` | `prometheus 0.14` | Metrics framework |

Nuevo default propuesto:

```
default = ["cli", "fjall", "sysinfo", "memmap2", "fs2", "rayon"]
```

### Blast radius

**Crates que dependen de `vantadb` con default features (se rompen):**

| Crate | Dependencia | Impacto |
|-------|-------------|---------|
| `vantadb-enterprise` | `vantadb = { path = ".." }` (hereda default) | **SE ROMPE** — usa default features implícitamente |
| Tests y benchmarks existentes | Usan `cargo test --workspace` sin `--features` | **SE ROMPEN** si necesitan arrow/rocksdb |

**Crates que NO se rompen (ya especifican features):**

| Crate | Features explicitas |
|-------|-------------------|
| `vantadb-python` | `default-features = false, features = ["fjall", "memmap2"]` |
| `vantadb-langchain` | `default-features = false, features = ["fjall", "memmap2"]` |
| `vantadb-ollama` | `default-features = false, features = ["fjall", "memmap2"]` |
| (y todos los adapters similares) | |
| `vantadb-server` | `features = ["cli", "server"]` (no hereda default) |
| `vantadb-mcp` | `features = ["cli"]` (no hereda default) |
| `vantadb-wasm` | `default-features = false, features = ["wasm"]` |

**CI impactado:**

| Workflow | Cambio necesario |
|----------|-----------------|
| `ci-rust-10.yml` test Linux | Ya usa `--features "cli,arrow,tls,opentelemetry"` → **OK** (sigue compilando arrow) |
| `ci-rust-10.yml` test Windows | Ya usa `--features "cli,arrow,tls,opentelemetry"` → **OK** |
| `ci-rust-10.yml` clippy | Usa `--all-features` → **OK** |
| `ci-rust-10.yml` coverage | Usa `--features "cli,arrow,tls,opentelemetry"` → **OK** |
| `ci-rust-10.yml` MSRV | Usa `cargo check --workspace` → **ROMPE** si enterprise depende de default |
| `ci-rust-10.yml` minimal-versions | Usa `cargo check --workspace` → **ROMPE** idem |

**verify.ps1 / verify_changed.ps1:**

| Script | Cambio necesario |
|--------|-----------------|
| `verify.ps1` | `cargo check --workspace --tests` → igual funciona, pero enterprise se rompe |
| `verify_changed.ps1` | Idem |

### Plan de acción

1. **Primero**: Parchear `vantadb-enterprise/Cargo.toml` para que especifique features explícitamente:
   ```toml
   vantadb = { path = "..", features = ["fjall"] }
   # enterprise usa solo config.rs + VantaEmbedded que están en core
   ```

2. **Segundo**: Modificar `Cargo.toml:86`:
   ```toml
   default = ["cli", "fjall", "sysinfo", "memmap2", "fs2", "rayon"]
   ```

3. **Tercero**: Verificar que los tests que dependen de arrow/rocksdb compilen en CI con las features correctas

4. **Cuarto**: En `verify.ps1`, asegurar que `cargo check --workspace` funcione (enterprise ya no rompe)

### Post-tarea

- [ ] `vantadb-enterprise` compila con `cargo check -p vantadb-enterprise`
- [ ] `cargo check --workspace` compila completo
- [ ] CI `ci-rust-10.yml` todos los jobs pasan
- [ ] `cargo test -p vantadb --features arrow` funciona (para quien necesite arrow)
- [ ] `cargo check -p vantadb --no-default-features -F fjall` es notablemente más rápido

---

## Tarea 6 — Separar adapters en otro workspace

### Objetivo

Sacar las 10 crates adapter del workspace principal para que `cargo check --workspace` sea mucho más rápido.

### Diagnóstico

Los adapters son crates que:
- Raramente cambian (son thin wrappers de pyo3 sobre VantaEmbedded)
- Dependen de `vantadb` con `default-features = false, features = ["fjall", "memmap2"]`
- Todos son `cdylib` (pyo3 shared library) — cada uno compila pyo3 separadamente
- Cada adapter compila pyo3, serde, serde_json, etc.

Lista de adapters a mover:
1. `vantadb-mem0`
2. `vantadb-letta`
3. `vantadb-crewai`
4. `vantadb-dspy`
5. `vantadb-haystack`
6. `vantadb-litellm`
7. `vantadb-openai`
8. `vantadb-ollama`
9. `vantadb-langchain`
10. `vantadb-llamaindex`

Crates que **NO** se mueven:
- `vantadb-server` — se toca frecuentemente, depende de `vantadb-mcp`
- `vantadb-mcp` — se toca frecuentemente
- `vantadb-python` — se toca frecuentemente
- `vantadb-wasm` — no es adapter, tiene su propio build
- `vantadb-enterprise` — aunque raramente se toca, depende de default features

### Blast radius

**CI impactado:**

| Workflow | Cambio necesario |
|----------|-----------------|
| `ci-rust-10.yml` (todos los jobs) | Usan `--workspace` + `vantadb-*/**` en paths trigger. Adaptadores ya no están en workspace → CI no los testea |
| `release-wheels-60.yml` | Usa `cargo test --test version_coherence --no-default-features --features fjall` → **OK** (no usa adapters) |
| `heavy-certification-50.yml` | No usa workspace → **OK** |

**Integraciones Python:**
- `integrations/` contiene wrappers Python (LangChain, LlamaIndex) que referencian a los adapters Rust
- Esos wrappers usan `maturin` o `pip install` de los adapters
- Si los adapters están fuera del workspace, las integraciones siguen funcionando porque usan path dependency a `vantadb`

**verify.ps1 / verify_changed.ps1:**
- Ambos usan `cargo check --workspace` → ya no incluyen adapters. **Intencional.**
- Los adapters tienen tests Python propios. Habría que correrlos explícitamente si se cambia `vantadb` core.

**API breaks:**
- Si cambia la API pública de `vantadb` (VantaEmbedded, VantaError, etc.), los adapters fuera del workspace no se romperán en `cargo check` local
- Pero sí en CI de los adapters (cuando exista) o cuando alguien haga `cargo build -p vantadb-ollama`

**Solución al risk de API breaks:**
- Opción A: Dejar los adapters en el workspace pero excluirlos de `cargo check --workspace` con `--exclude`
- Opción B: Crear un CI job separado que haga `cargo check` de los adapters contra `path = ".."` (sigue funcionando como path dep aunque no estén en el workspace)

### Plan de acción

**Opción simple (recomendada):** NO sacar del workspace. En su lugar, modificar los comandos para excluirlos.

```bash
# En lugar de:
cargo check --workspace

# Usar:
cargo check --workspace \
  --exclude vantadb-mem0 \
  --exclude vantadb-letta \
  --exclude vantadb-crewai \
  --exclude vantadb-dspy \
  --exclude vantadb-haystack \
  --exclude vantadb-litellm \
  --exclude vantadb-openai \
  --exclude vantadb-ollama \
  --exclude vantadb-langchain \
  --exclude vantadb-llamaindex
```

Esto es más simple, no requiere mover archivos, no rompe CI, y da el mismo beneficio.

**Opción avanzada (separar workspace):**
1. Crear `adapters/Cargo.toml`:
   ```toml
   [workspace]
   members = [
       "vantadb-mem0",
       "vantadb-letta",
       ...
   ]
   ```
2. Mover cada adapter a `adapters/vantadb-{nombre}/`
3. Cambiar path dependency de `vantadb = { path = "../.." }`
4. Agregar CI job separado para los adapters
5. Quitar del workspace principal

### Post-tarea

- [ ] `cargo check --workspace --exclude ...` compila en <50% del tiempo original
- [ ] CI pasa en todos los jobs
- [ ] Los adapters individuales compilan con `cargo check -p vantadb-ollama`
- [ ] `verify.ps1` actualizado si se opta por --exclude list

---

## Tarea 7 — Consolidar deps duplicadas

### Objetivo

Reducir cantidad de versiones duplicadas del mismo crate para que cargo compile menos código redundante.

### Diagnóstico

Actualmente hay 27 crates con versiones duplicadas. Las más relevantes:

| Crate | Versiones | Impacto |
|-------|-----------|---------|
| `hashbrown` | 0.14.x, 0.15.x, 0.5.x | 4 versiones compiladas |
| `windows-sys` | 0.52.x, 0.59.x, 0.60.x, 0.61.x | 4 versiones |
| `getrandom` | 0.2.x, 0.3.x, 0.4.x | 3 versiones |
| `thiserror` | 1.x, 2.x | 2 versiones |
| `reqwest` | 0.12.x, 0.13.x | 2 versiones |
| `rand` + `rand_core` | 0.8.x, 0.9.x | 2 versiones cada uno |
| `itertools` | 2 versiones | |
| `lru` | 2 versiones | |
| `shlex` | 1.3, 2.0 | |

Cada versión extra se compila separadamente y ocupa espacio en disco + tiempo de compilación.

### Plan de acción

1. Identificar qué crates traen cada versión vieja:
   ```bash
   cargo tree -d
   ```

2. Para cada duplicado:
   - Si la versión vieja viene de una dependencia directa: actualizar Cargo.toml
   - Si viene de una dependencia transitiva: ver si la dep upstream ya actualizó. Si no, PR al upstream o fork
   - Ejecutar `cargo update` para consolidar semver-compatibles

3. Casos especiales:
   - `thiserror 1.x` → puede venir de deps que no migraron a 2.x. Ver si hay breaking
   - `reqwest 0.12` → 0.13 puede tener breaking changes
   - `hashbrown` versiones viejas → vienen de hashbrown 0.14/0.15 que son compatibles

### Blast radius

| Ítem | Impacto |
|------|---------|
| **thiserror 1.x → 2.x** | Breaking si hay pattern matching en `Error` variants. Revisar si alguna dependencia usa `thiserror::Error` de 1.x |
| **reqwest 0.12 → 0.13** | Breaking si se usa API deprecada. `vantadb` usa reqwest 0.12 como dep opcional |
| **hashbrown 0.14 → 0.15** | Compatible en el 99% de los casos |
| **windows-sys** | Compatible entre versiones (no se linkea directamente, solo `use`) |
| **getrandom** | Compatible. La versión 0.4 es más portable (incluye wasm). La 0.2/0.3 son legacy |
| **rand 0.8 → 0.9** | Breaking: 0.9 cambió la API de generadores. `vantadb` ya usa rand 0.9. La versión 0.8 viene de transitivas |

### Post-tarea

- [ ] `cargo check --workspace` compila sin errores
- [ ] `cargo nextest run --profile audit --workspace` pasa
- [ ] `cargo tree -d` muestra menos duplicados que antes (ideal: 0)
- [ ] Actualizar `Cargo.lock`

---

## Resumen de Scripts/Archivos que TOCA CADA TAREA

| Tarea | Archivos a modificar |
|-------|---------------------|
| **T1** Profile check | ❌ No aplicable (nombre `check` reservado, el built-in ya es óptimo) |
| **T2** Quitar target-cpu | `.cargo/config.toml` (comentar `rustflags`) |
| **T3** Podar deps | Varios `Cargo.toml` (eliminar deps), opcional agregar `ignored` a machete |
| **T4** sccache | `.cargo/config.toml` (opcional), perfil de PowerShell (recomendado) |
| **T5** Features heavies | `Cargo.toml` (default features), `vantadb-enterprise/Cargo.toml` (features explícitas) |
| **T6** Separar adapters | `Cargo.toml` (workspace members), `dev-tools/verify.ps1`, `dev-tools/verify_changed.ps1`, Justfile, CI workflows, o crear `adapters/` workspace |
| **T7** Consolidar deps | `Cargo.toml` (upgrade versiones), `Cargo.lock` (cargo update) |

---

## Orden de Ejecución Recomendado

```
Fase 1 (5 min c/u, riesgo bajo, impacto inmediato):
  T2 → T3 → T4

Fase 2 (1-2 hr, requiere verificación en CI):
  T5 → T7

Fase 3 (2-4 hr, requiere planificación):
  T6
```

Cada fase deja el proyecto compilando y los tests pasando antes de pasar a la siguiente.

---

## Resultados de Tareas Ejecutadas

### ✅ T2 ejecutada (2026-07-13)

**Cambio aplicado:** En `.cargo/config.toml` se comentó `target-cpu=native` y se reemplazó con `rustflags = []`.

**Archivo:** `.cargo/config.toml` (no trackeado por git — [.gitignore linea 109](../.gitignore)). El cambio es local.

**Verificación:** `cargo check -p vantadb --no-default-features -F "fjall,cli"` compila en ~38s con 0 errores nuevos.

**Rollback:** Descomentar la línea `rustflags = ["-C", "target-cpu=native"]` y eliminar `rustflags = []`.

---

### ❌ T1 no aplicable

`cargo check` ya usa el profile built-in `check` con `opt-level=0, debug=0, codegen-units=256`. No hay configuración que mejore eso. El nombre `check` es reservado por Cargo y no se puede crear `[profile.check]`.

---

## Verificación Final (después de TODAS las tareas)

- [ ] `cargo check --workspace` compila
- [ ] `cargo nextest run --profile audit --workspace` pasa
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` pasa
- [ ] CI `ci-rust-10.yml` pasa en todos los jobs
- [ ] `cargo check -p vantadb --no-default-features -F fjall` es notablemente más rápido que antes
- [ ] Medir y documentar la mejora en tiempo (`Measure-Command { cargo check -p vantadb }`)
