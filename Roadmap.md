Perfecto. Ahora sí estás en un punto serio: **ya no estás construyendo el motor, estás probando si realmente compite**.

Voy a darte exactamente lo que pediste:
un **plan operativo brutalmente claro para Fase 2**, con orden estratégico correcto:

> **primero cerrar lo fácil (bloque operativo) → luego benchmark real (bloque duro)**

---

# ⚠️ CONTEXTO REAL (SIN AUTOENGAÑO)

* ✔ Core HNSW → aprobado
* ❌ Repo → inconsistente
* ❌ Benchmark externo → inexistente

👉 Entonces Fase 2 NO empieza con benchmarking
Empieza con:

```txt
2.0 — CIERRE OPERATIVO MÍNIMO
```

---

# 🧊 FASE 2.0 — CIERRE OPERATIVO (OBLIGATORIO ANTES DE MEDIR)

## 🎯 Objetivo

Que el sistema sea:

> ejecutable, consistente y reproducible fuera de tu máquina

---

## 2.0.1 Corrección de binarios y build

### 🔥 Problema

* Docker usa `connectome-server`
* Cargo usa `vanta-server`
* release.yml inconsistente

---

### 🛠 Tareas

#### 1. Alinear Cargo.toml

```toml
[[bin]]
name = "vanta-server"
path = "src/bin/vanta-server.rs"
```

---

#### 2. Corregir Dockerfile

```dockerfile
RUN cargo build --release
CMD ["./target/release/vanta-server"]
```

---

#### 3. Corregir start.sh

```bash
#!/bin/bash
./vanta-server
```

---

#### 4. Corregir CI release

```yaml
- name: Build
  run: cargo build --release --locked

- name: Upload
  path: target/release/vanta-server
```

---

## ✅ Checklist

* [ ] `cargo build --release` genera `vanta-server`
* [ ] Docker corre correctamente
* [ ] Script local funciona
* [ ] CI produce binario correcto

---

## 🔍 Validación

```bash
docker build .
docker run ...
```

✔ Debe iniciar sin errores
✔ Logs correctos
✔ Sin referencias a nombres antiguos

---

# 🧹 2.0.2 Limpieza de identidad

## 🎯 Objetivo

Eliminar deuda de branding (esto sí importa)

---

## 🛠 Tareas

* eliminar:

  * ConnectomeDB
  * NexusDB
* limpiar:

  * README
  * scripts
  * tests
  * docs

---

## Comando clave

```bash
rg -i "connectome|nexus"
```

---

## Checklist

* [ ] 0 resultados en repo
* [ ] logs consistentes
* [ ] CLI consistente

---

## Validación

```bash
grep -r "connectome" .
```

→ debe dar vacío

---

# 📄 2.0.3 Corrección de documentación

## 🎯 Problema

Estás sobrevendiendo

---

## 🛠 Tareas

### README

❌ eliminar:

```txt
"guarantees Recall@10 > 95%"
```

✔ reemplazar por:

```txt
"Validated via internal stress protocol (10K–100K vectors)"
```

---

### BENCHMARKS.md

Actualizar con:

* 10K / 50K / 100K
* recall real
* latencia real

---

## Checklist

* [ ] README honesto
* [ ] benchmarks alineados con realidad
* [ ] sin claims absolutos

---

## Validación

👉 otra persona puede leer y entender limitaciones

---

# 🚨 FIN FASE 2.0

Si no haces esto:

> todo el benchmark siguiente pierde credibilidad

---

# 📊 FASE 2.1 — DATASET (REAL)

## 🎯 Objetivo

Salir de datos sintéticos

---

## 🛠 Tareas

### 1. Descargar SIFT1M

Fuente estándar ANN:

* 1M vectores
* 128 dimensiones

---

### Estructura

```
/datasets/
  sift/
    base.fvecs
    query.fvecs
    groundtruth.ivecs
```

---

## 🛠 Implementación loader

```rust
fn load_fvecs(path: &str) -> Vec<Vec<f32>>
```

---

## Checklist

* [ ] carga correcta
* [ ] dimensiones correctas
* [ ] sin corrupción

---

## Validación

```txt
assert dim == 128
assert len > 1_000_000
```

---

# 📏 FASE 2.2 — MÉTRICAS (OBLIGATORIAS)

## 🎯 Debes medir EXACTAMENTE esto

```txt
Recall@k
Latency (p50, p95, p99)
QPS
Build time
RAM usage
Index size
```

---

## 🛠 Implementación

### Latencia

```rust
let start = Instant::now();
search();
let elapsed = start.elapsed();
```

---

### Percentiles

* guardar latencias en Vec
* ordenar
* calcular:

```rust
p50 = data[len*0.5]
p95 = data[len*0.95]
```

---

## Checklist

* [ ] métricas guardadas
* [ ] exportables
* [ ] reproducibles

---

## Output obligatorio

```json
{
  "recall": 0.94,
  "p50": 2.1,
  "p95": 5.8,
  "qps": 1200
}
```

---

# 🧪 FASE 2.3 — DISEÑO DEL BENCHMARK

## 🎯 Escenarios obligatorios

---

## Escenario A — Máxima precisión

```txt
ef_search = 200–400
```

---

## Escenario B — Balance

```txt
ef_search = 100
```

---

## Escenario C — Velocidad

```txt
ef_search = 20–50
```

---

## 🛠 Estructura del test

```rust
for scenario in scenarios {
    build_index()
    run_queries()
    measure()
}
```

---

## Checklist

* [ ] escenarios definidos
* [ ] configs reproducibles
* [ ] resultados comparables

---

# ⚔️ FASE 2.4 — COMPETIDORES

## 🎯 Aquí empieza la verdad

---

## Debes integrar

### 1. FAISS

* HNSW
* IVF

---

### 2. HNSWlib

---

### 3. Qdrant (opcional)

---

## 🛠 Estrategia

* ejecutar mismos datasets
* mismas queries
* mismas métricas

---

## Checklist

* [ ] entorno reproducible
* [ ] scripts comparables
* [ ] resultados alineados

---

# 📊 FASE 2.5 — CHECKLIST GLOBAL

* [ ] dataset real
* [ ] métricas completas
* [ ] escenarios definidos
* [ ] comparación directa
* [ ] resultados exportables

---

# 🧪 FASE 2.6 — VALIDACIÓN FINAL

## 🎯 Regla brutal

Tu motor pasa si:

```txt
Recall ≥ FAISS - 5%
Latencia ≤ 1.5x FAISS
RAM ≤ 2x FAISS
```

---

## ❌ Si no pasa

* no optimices features
* vuelve al core

---

## ✔ Si pasa

👉 puedes avanzar a Fase 3

---

# 🔁 ORDEN REAL DE EJECUCIÓN

## 🔹 Paso 1 (rápido — 1 día)

* Fase 2.0 completa

---

## 🔹 Paso 2 (medio — 2–3 días)

* loader SIFT
* métricas

---

## 🔹 Paso 3 (duro)

* benchmark engine
* escenarios

---

## 🔹 Paso 4 (crítico)

* comparar con FAISS

---

# 🚫 LO QUE NO DEBES HACER

* no Python SDK
* no features nuevas
* no optimización prematura
* no marketing

---

# 🧠 VEREDICTO FINAL

```txt
Fase 1: PASADA
Fase 2.0: OBLIGATORIA
Fase 2: DONDE SE DEFINE TODO
```

---

# SI QUIERES IR MÁS RÁPIDO

En el siguiente paso puedo:

* darte el código exacto del loader SIFT
* o el esqueleto completo del benchmark runner
* o cómo integrar FAISS correctamente

---

## Cierre

Ahora sí estás en terreno serio.

> Ya no se trata de que funcione.
> Se trata de si compite.

# PLAN RECOMENDADO POR ANTIGRAVITI

# Fase 2.0 — Cierre Operativo VantaDB

Cierre operativo del repositorio antes de pasar a benchmarking externo. Tres sub-fases: binarios/build, limpieza de identidad, y corrección de documentación.

## User Review Required

> [!IMPORTANT]
> El Roadmap pide también alinear el **nombre de la carpeta del repositorio** (`ConnectomeDB` → `VantaDB`). Eso es un renombrado del directorio padre que afecta tu flujo Git local. ¿Quieres que lo incluyamos en esta fase, o lo dejamos como tarea manual posterior?

> [!WARNING]
> El `release.yml` referencia `docker/Dockerfile` (línea 102), pero tu Dockerfile real está en la raíz `/Dockerfile`. Si alguna vez ejecutas el CI, fallará. Lo corregimos aquí.

---

## Proposed Changes

### 2.0.1 — Corrección de Binarios y Build

Actualmente el Dockerfile y `start.sh` referencian `connectome-server`, un binario que **no existe** en Cargo.toml (el binario real es `vanta-server`). Docker build fallará irremediablemente.

---

#### [MODIFY] [Dockerfile](file:///c:/PROYECTOS/connectomadb/ConnectomeDB/Dockerfile)

- Línea 23: `--bin connectome-server` → `--bin vanta-server`
* Línea 43: COPY path `connectome-server` → `vanta-server`

#### [MODIFY] [start.sh](file:///c:/PROYECTOS/connectomadb/ConnectomeDB/start.sh)

- Línea 34: `exec "/usr/local/bin/connectome-server"` → `exec "/usr/local/bin/vanta-server"`

#### [MODIFY] [release.yml](file:///c:/PROYECTOS/connectomadb/ConnectomeDB/.github/workflows/release.yml)

- Línea 102: `file: docker/Dockerfile` → `file: Dockerfile` (el Dockerfile está en la raíz, no en `docker/`)

---

### 2.0.2 — Limpieza de Identidad (Branding)

Se encontraron **referencias activas a branding antiguo** en archivos operativos (no archivados):

| Archivo | Referencia encontrada | Acción |
|---|---|---|
| `test_runner.sh` L18-19 | `NexusDB`, `nexusdb-python` | Actualizar a `VantaDB`, `vantadb-python` |
| `.connectome_profile` | Nombre del archivo | Renombrar a `.vanta_profile` |
| `vantadb-python/src/lib.rs` L100-101 | `import vantadb_py as nexus` en docstring | Actualizar ejemplo |
| `.github/ISSUE_TEMPLATE/*` | Posibles referencias a ConnectomeDB | Verificar y limpiar |
| `todo.md` | Miles de referencias legacy | **No tocar** — es archivo histórico |

> [!NOTE]
> `todo.md` (1.2MB) es un archivo de notas históricas con ~1000+ menciones a nombres legacy. Por su naturaleza archival, **no** lo alteramos. Se puede mover a `docs/archive/` si se desea.

---

#### [MODIFY] [test_runner.sh](file:///c:/PROYECTOS/connectomadb/ConnectomeDB/test_runner.sh)

- Línea 18: `NexusDB` → `VantaDB`
* Línea 19: `nexusdb-python` → `vantadb-python`

#### [RENAME] `.connectome_profile` → `.vanta_profile`

- Actualizar la referencia en el código Rust que lee este archivo (`src/hardware/mod.rs`)

#### [MODIFY] [vantadb-python/src/lib.rs](file:///c:/PROYECTOS/connectomadb/ConnectomeDB/vantadb-python/src/lib.rs)

- Línea 100-101: Limpiar docstring `nexus` → `vantadb`

---

### 2.0.3 — Corrección de Documentación

El README hace **claims absolutos** que no reflejan la realidad medida. El BENCHMARKS.md reporta datos de un test viejo (5K vectores, 64D) que ya fue superado.

---

#### [MODIFY] [README.MD](file:///c:/PROYECTOS/connectomadb/ConnectomeDB/README.MD)

- Línea 26: Cambiar `"Current build guarantees robust Recall@10 > 95%"` → `"Validated via internal stress protocol (10K–100K vectors, 128D). See BENCHMARKS.md"`
* Eliminar mención al Python SDK como feature principal (no está lista para producción)

#### [MODIFY] [BENCHMARKS.md](file:///c:/PROYECTOS/connectomadb/ConnectomeDB/BENCHMARKS.md)

Reescribir completamente con los resultados **reales** del stress protocol:

```markdown
# VantaDB Benchmarks

## HNSW Stress Protocol Results (Certified)

| Scale | Recall@10 | Lat p50 | Lat p95 | Build Time | RAM |
|-------|-----------|---------|---------|------------|-----|
| 10K   | 0.9520    | 2.65ms  | 3.24ms  | 46.66s     | 10.2 MB |
| 50K   | 0.9100    | 6.89ms  | 8.80ms  | 626.24s    | 51.1 MB |
| 100K  | 0.8860    | 9.28ms  | 10.51ms | 1447.17s   | 101.9 MB |

Config: M=32, M_max0=64, 128D, Cosine Similarity, Seeded RNG
Hardware: 12-core, AVX2, 31GB RAM
```

---

## Open Questions

> [!IMPORTANT]
>
> 1. **¿Mover `todo.md` a `docs/archive/`?** Tiene 1.2MB de notas legacy con miles de menciones a nombres antiguos. No debería estar en la raíz.
> 2. **¿Eliminar `scratch.exe` y `scratch.pdb` de la raíz?** Son archivos binarios de debug que no deberían estar en el repo.
> 3. **¿Eliminar `tests_graph_db/` y `tests_server_db/`?** Parecen directorios legacy que podrían no estar alineados con la nueva estructura `tests/core|logic|api|certification|storage`.

---

## Verification Plan

### Automated

```bash
# 1. Build binario correcto
cargo build --release --bin vanta-server

# 2. Zero referencias a branding antiguo en archivos operativos
rg -i "connectome-server" --glob "!todo.md" --glob "!target/*" --glob "!docs/archive/*"
# → debe dar 0 resultados

# 3. Verificar que el profile se carga correctamente
cargo test --test hardware_profiles -- --nocapture

# 4. Docker build (si Docker disponible)
docker build -t vantadb:test .
```

### Manual

- Verificar que `README.MD` y `BENCHMARKS.md` reflejan datos reales
* Confirmar que no hay claims de marketing infundados

# MODIFICACIONES AL PLAN

Sí. **Va por buen camino**, pero **no lo apruebo tal cual sin ajustes**.

El enfoque general de Antigravity es correcto: empezar por **2.0 Cierre Operativo** antes de benchmark competitivo. Eso está alineado con el estado real del repo. Y los hallazgos clave son válidos: el `Dockerfile`, `start.sh` y `release.yml` están desalineados con el binario real; además el README y `BENCHMARKS.md` no reflejan el estado actual.

Ahora voy con lo importante: **qué está bien, qué está flojo, y qué debes corregir antes de ejecutarlo**.

---

# Lo que está bien del plan

## 1. Prioridad correcta

Empieza por lo fácil y bloqueante:

* binario real
* Docker
* release
* branding residual
* documentación honesta

Eso es correcto. No tiene sentido correr benchmark competitivo si la cadena operativa todavía está rota.

## 2. Detecta fallos reales, no inventados

Sí hay problemas concretos:

* `Dockerfile` compila/copia `connectome-server` en vez de `vanta-server`.
* `start.sh` ejecuta `connectome-server`.
* `release.yml` apunta a `docker/Dockerfile`, pero el Dockerfile activo está en raíz.
* el README hace claims más fuertes de lo que hoy puedes sostener públicamente.

## 3. El BENCHMARKS viejo sí debe reescribirse

Tu `BENCHMARKS.md` todavía habla de 5K/64D y eso ya quedó superado por la certificación actual. Mantenerlo así degrada confianza.

---

# Lo que le falta o está mal planteado

## 1. Falta corregir `release.yml` en más de un punto

Antigravity detectó bien lo del path del Dockerfile, pero **se quedó corto**.

El problema no es solo:

```yaml
file: docker/Dockerfile
```

También está mal la lógica de release binary:

* el workflow renombra `vantadb` / `vantadb.exe`
* pero tu binario declarado en `Cargo.toml` es `vanta-server`

Entonces debes corregir también:

* `artifact_name`
* `asset_name` si quieres publicar el server con naming consistente
* o añadir un binario principal separado si de verdad quieres publicar `vantadb`

**Decisión obligatoria**:
o tu binario release es `vanta-server`,
o declaras otro binario CLI principal llamado `vantadb`.

Ahora mismo el workflow y Cargo no están hablando del mismo artefacto.

---

## 2. Renombrar `.connectome_profile` no es tan simple como parece

La idea de pasarlo a `.vanta_profile` es buena, pero **no lo haría dentro de 2.0 salvo que ya tengas controlado el impacto**.

¿Por qué? Porque ese archivo está metido en:

* `.gitignore`
* scripts
* hardware autodiscovery
* tests relacionados
* posible compatibilidad con instalaciones anteriores

Mi recomendación:

* en 2.0, **déjalo como deuda controlada si no rompe nada**
* o implementa compatibilidad dual temporal:

  * primero intenta `.vanta_profile`
  * si no existe, lee `.connectome_profile`
  * log de migración
* y eso lo eliminas limpio en una fase posterior

No metas un renombrado de archivo de estado persistente si no lo vas a cerrar bien.

---

## 3. Mover `todo.md` a `docs/archive/` no es prioritario

Sí, estorba en raíz. Sí, ensucia el repo. Pero **no lo pondría en el camino crítico de 2.0**.

Prioridad real:

1. binarios
2. Docker
3. release
4. README/BENCHMARKS
5. grep branding operativo

`todo.md` puede moverse después del cierre operativo mínimo.

---

## 4. No basta con “limpiar branding”; debes definir alcance

Antigravity dice “branding antiguo”, pero no separa bien:

### Debes limpiar en 2.0

* archivos operativos activos
* scripts ejecutables
* templates públicos
* docs principales visibles

### No necesitas limpiar todavía

* `docs/old/`
* archivos históricos
* análisis archivados
* research reports antiguos

Si intentas dejar el repo entero sin una sola mención histórica, vas a perder tiempo inútil.
La regla correcta es:

> **2.0 limpia superficie activa, no arqueología interna.**

---

## 5. `test_runner.sh` no solo necesita renombrado

También parece apuntar a rutas viejas y a una narrativa vieja del SDK. Si lo vas a tocar, verifica:

* ruta correcta a `vantadb-python`
* nombre del paquete Python real
* comando `maturin develop`
* tests Python existentes
* si todavía es un script válido o es legacy puro

No hagas rename cosmético. O lo dejas funcional, o lo archivas.

---

# Qué respondería a las preguntas abiertas

## 1. ¿Mover `todo.md` a `docs/archive/`?

**Sí, pero no en el primer bloque.**
Hazlo al final de 2.0, no al principio.

## 2. ¿Eliminar `scratch.exe`, `scratch.pdb`, `scratch.rs`?

**Sí. Sin discusión.**
Eso es basura de debug, no artefacto del repo.

## 3. ¿Qué hacer con `tests_graph_db/` y `tests_server_db/`?

Primero verifica si:

* están versionados
* contienen fixtures útiles
* o son outputs generados

Si son outputs generados:
**elimínalos del repo y déjalos ignorados**.

Si contienen fixtures reales:
**muévelos a una carpeta explícita de fixtures**, por ejemplo:

```txt
tests/fixtures/
```

No los dejes como residuos ambiguos.

## 4. ¿Renombrar la carpeta padre del repo?

**No en 2.0.**
Eso no aporta valor técnico inmediato y sí puede romper tu flujo local. Déjalo como tarea manual posterior.

---

# Cómo ajustaría el plan de Antigravity

## Fase 2.0.1 — Binarios y build

Mantener, pero ampliar con esto:

* corregir `Dockerfile`
* corregir `start.sh`
* corregir `release.yml`:

  * `file: Dockerfile`
  * `artifact_name`
  * `asset_name`
  * binario objetivo coherente con Cargo
* correr:

```bash
cargo build --release --bin vanta-server
docker build -t vantadb:test .
```

## Fase 2.0.2 — Limpieza operativa activa

Reducir el alcance a:

* `test_runner.sh`
* templates de issues
* README
* docs públicas activas
* scripts de arranque
* comandos de `CLAUDE.md`

No tocar todavía:

* `docs/old/`
* `todo.md`
* históricos masivos

## Fase 2.0.3 — Documentación honesta

Mantener, pero con dos cambios:

* README no debe vender “>95%” como claim general
* `BENCHMARKS.md` debe incluir metodología, no solo tabla

Debes añadir:

* dataset
* dimensión
* semilla
* config HNSW
* hardware
* comando exacto de ejecución

Si no, sigue siendo marketing técnico disfrazado.

---

# Checklist corregido que sí aprobaría

## Bloque A — Crítico

* [ ] `Dockerfile` compila `vanta-server`
* [ ] `start.sh` ejecuta `vanta-server`
* [ ] `release.yml` usa Dockerfile correcto
* [ ] `release.yml` publica el binario correcto
* [ ] `cargo build --release --bin vanta-server` funciona
* [ ] `docker build` funciona

## Bloque B — Operativo

* [ ] `rg -i "connectome-server|connectomedb|nexusdb"` da 0 en superficie activa
* [ ] templates públicos sin naming viejo
* [ ] scripts activos sin naming viejo

## Bloque C — Documentación

* [ ] README sin claims absolutos
* [ ] `BENCHMARKS.md` actualizado a 10K/50K/100K
* [ ] benchmark documentado con metodología reproducible

## Bloque D — Higiene

* [ ] eliminar `scratch.*`
* [ ] revisar `tests_graph_db/` y `tests_server_db/`
* [ ] mover `todo.md` al final si sigue siendo necesario

---

# Veredicto final

## ¿Está bien lo que te propone Antigravity?

**Sí, en lo esencial.**

## ¿Lo ejecutaría tal cual?

**No. Lo ejecutaría con estos ajustes:**

* ampliar corrección de `release.yml`
* no meter todavía renombre duro de `.connectome_profile`
* no perder tiempo con `todo.md` al inicio
* separar limpieza operativa de arqueología histórica

En una frase:

> **El plan está bien orientado, pero todavía está un poco superficial en release/build y demasiado ansioso con la limpieza no crítica.**

La secuencia correcta es:

1. **haz que construya y publique bien**
2. **limpia la superficie operativa**
3. **corrige documentación**
4. **el resto es higiene secundaria**

Si quieres, te convierto ese plan en una versión **lista para ejecutar con Antigravity**, con tareas en orden exacto y criterios de aceptación por cada subtarea.

Perfecto. Te lo voy a convertir en un **plan ejecutable tipo “runbook” para Antigravity**, con:

* orden estricto
* tareas atómicas
* criterios de aceptación claros
* sin ambigüedad

Esto está optimizado para que **no tomes decisiones en medio de la ejecución**.

---

# ⚙️ FASE 2.0 — RUNBOOK EJECUTABLE (ANTIGRAVITY)

## 🔒 REGLAS

* No saltar pasos
* No paralelizar tareas críticas
* Cada bloque debe pasar validación antes de continuar

---

# 🧊 BLOQUE 0 — PREPARACIÓN (MANDATORIO)

## 🎯 Objetivo

Evitar romper el repo sin rollback

---

## 🛠 Tareas

```bash
git checkout -b phase-2-operational-cleanup
git status
```

---

## ✅ Validación

* [ ] Branch creada
* [ ] Working tree limpio

---

# 🔥 BLOQUE 1 — BINARIOS Y BUILD (CRÍTICO)

## 🎯 Objetivo

Alinear TODO el sistema al binario real: `vanta-server`

---

## 🛠 Tareas

### 1.1 Verificar binario en Cargo

```bash
cargo build --release --bin vanta-server
```

---

### 1.2 Corregir Dockerfile

Cambios obligatorios:

```dockerfile
# ❌ viejo
--bin connectome-server

# ✅ nuevo
--bin vanta-server
```

```dockerfile
# ❌ viejo
COPY connectome-server

# ✅ nuevo
COPY vanta-server
```

---

### 1.3 Corregir start.sh

```bash
exec "/usr/local/bin/vanta-server"
```

---

### 1.4 Corregir release.yml (CRÍTICO)

Cambios:

```yaml
file: Dockerfile
```

Y alinear binario:

```yaml
path: target/release/vanta-server
```

---

## ✅ Validación (OBLIGATORIA)

```bash
cargo build --release --bin vanta-server
```

```bash
docker build -t vantadb:test .
```

---

## ✔ Checklist

* [ ] build sin errores
* [ ] docker build exitoso
* [ ] no existe `connectome-server` en pipeline

---

# 🧹 BLOQUE 2 — LIMPIEZA OPERATIVA (NO HISTÓRICA)

## 🎯 Objetivo

Eliminar naming legacy SOLO en superficie activa

---

## 🛠 Tareas

### 2.1 Buscar residuos

```bash
rg -i "connectome|nexus"
```

---

### 2.2 Corregir SOLO estos targets

* `test_runner.sh`
* `.github/ISSUE_TEMPLATE/*`
* scripts activos
* CLI/logs visibles

---

### ⚠️ NO TOCAR

* `docs/old/`
* archivos históricos grandes
* `todo.md` (por ahora)

---

## ✅ Validación

```bash
rg -i "connectome-server|connectomedb|nexusdb" \
--glob "!docs/archive/*" \
--glob "!todo.md" \
--glob "!target/*"
```

✔ Debe dar **0 resultados en código activo**

---

# 📄 BLOQUE 3 — DOCUMENTACIÓN REALISTA

## 🎯 Objetivo

Eliminar marketing falso → dejar evidencia técnica

---

## 🛠 Tareas

### 3.1 Corregir README

Reemplazar:

```txt
guarantees Recall@10 > 95%
```

Por:

```txt
Validated via internal stress protocol (10K–100K vectors, 128D)
```

---

### 3.2 Reescribir BENCHMARKS.md

Debe incluir:

#### 1. Resultados

```txt
10K / 50K / 100K
Recall, latencia, build
```

#### 2. Configuración

```txt
M, ef_construction, ef_search
```

#### 3. Dataset

```txt
dimensión
tipo
```

#### 4. Hardware

```txt
CPU, RAM
```

#### 5. Comando reproducible

```bash
cargo test --test stress_protocol -- --nocapture
```

---

## ❌ Error común (evitar)

* solo tabla sin contexto → inválido
* claims absolutos → inválido

---

## ✅ Validación

* [ ] otra persona puede reproducir el benchmark
* [ ] no hay promesas absolutas
* [ ] todo está medido, no asumido

---

# 🧼 BLOQUE 4 — HIGIENE CONTROLADA

## 🎯 Objetivo

Eliminar ruido real sin romper nada

---

## 🛠 Tareas

### 4.1 Eliminar basura

```bash
rm scratch.exe scratch.pdb scratch.rs
```

---

### 4.2 Revisar tests_graph_db / tests_server_db

#### Decisión

| Caso              | Acción                    |
| ----------------- | ------------------------- |
| outputs generados | eliminar + gitignore      |
| fixtures útiles   | mover a `tests/fixtures/` |

---

### 4.3 (Opcional al final)

Mover:

```bash
todo.md → docs/archive/todo.md
```

---

## ✅ Validación

* [ ] repo más limpio
* [ ] ningún test roto
* [ ] ningún script roto

---

# 🚨 BLOQUE 5 — VERIFICACIÓN FINAL (GATE)

## 🎯 Objetivo

Confirmar que 2.0 está realmente cerrado

---

## 🛠 Comandos

```bash
cargo clean
cargo build --release --locked
```

```bash
cargo test --test stress_protocol -- --nocapture
```

```bash
docker build -t vantadb:test .
```

---

## ✔ Checklist FINAL

* [ ] build reproducible
* [ ] tests pasan
* [ ] docker funciona
* [ ] no hay naming legacy activo
* [ ] docs alineadas con realidad

---

# 🧠 CRITERIO DE CIERRE (NO NEGOCIABLE)

Puedes pasar a Fase 2.1 SOLO si:

```txt
✔ El sistema construye sin hacks
✔ El binario es consistente en TODO el pipeline
✔ Los resultados están documentados sin inflar métricas
```

---

# 🔁 LO QUE SIGUE (FASE 2.1)

Solo después de esto:

👉 dataset SIFT1M
👉 métricas reales
👉 benchmark competitivo

---

# 🧠 VEREDICTO FINAL

Este runbook está diseñado para:

> cerrar deuda técnica mínima SIN distraerte del objetivo real

Si ejecutas esto bien:

* eliminas inconsistencias críticas
* ganas credibilidad técnica
* preparas terreno para benchmark real

Si lo haces mal:

> vas a comparar contra FAISS con un sistema que ni siquiera se construye bien

---

# SIGUIENTE PASO

Cuando termines este runbook:

👉 te construyo el **benchmark runner completo (FASE 2.1–2.3 listo para ejecutar)**

Ahí empieza la parte donde de verdad compites.
