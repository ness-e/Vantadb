# Reorganización de Arquitectura — Implicaciones y Plan

**Goal:** Reorganizar la estructura del proyecto para eliminar duplicación, reducir confusión, y mover artefactos transientes fuera de la raíz, sin romper CI, builds, scripts, o documentación existente.

**Architecture:** 5 migraciones independientes que se ejecutan en orden específico para minimizar riesgo. Cada migración incluye: (1) mover archivos, (2) actualizar todas las referencias, (3) verificar que nada se rompe, (4) limpiar symlinks/stubs temporales.

**Tech Stack:** CI (GitHub Actions), Rust workspace (Cargo), Python (hatchling/setuptools), Documentación (Obsidian vault .md), Web (React/TanStack Router)

**Principio rector:** Cada migración DEBE ser reversible. No eliminamos nada en el primer paso — movemos y dejamos symlinks/redirects. Solo después de verificar que el ecosistema funciona, limpiamos.

---

### Mapa de dependencias entre migraciones

```
M1 (integrations) ──> M2 (benchmarks) ──> M3 (root artifacts) ──> M4 (docs/operations) ──> M5 (docs/reviews)
                                                                        │
                                                                        └──> M5 no depende de nadie
```

**M1** debe hacerse primero porque afecta CI triggers, rutas de importación Python, y empaquetado PyPI. **M4 y M5** son independientes y podrían ir en paralelo, pero por riesgo se hacen secuenciales.

---

## M1: Consolidar Integraciones

**Problema:** 3 ubicaciones para integraciones Python/Rust que se solapan conceptualmente:
- `integrations/langchain/` + `integrations/llamaindex/` (Python hatchling)
- `packages/langchain-vantadb/` + `packages/llamaindex-vantadb/` (Python setuptools)
- 8 crates `vantadb-haystack`, `vantadb-crewai`, `vantadb-dspy`, `vantadb-mem0`, `vantadb-letta`, `vantadb-litellm`, `vantadb-openai`, `vantadb-ollama` (Rust cdylib PyO3)

**⚠️ Hallazgo crítico:** `integrations/` y `packages/` NO son duplicados. Son implementaciones paralelas con APIs distintas, build systems distintos, dependencias distintas. **NO se puede simplemente eliminar una.** Hay dos estrategias posibles.

### Estrategia A (Recomendada): Unificar en `integrations/`

Convertir `integrations/` en el source of truth para los Python packages. Migrar `packages/langchain-vantadb/` y `packages/llamaindex-vantadb/` dentro de `integrations/` como directorios separados, renombrándolos para coexistir con los actuales.

```
Antes:
  integrations/
    langchain/          → package "vantadb-langchain" (hatchling)
    llamaindex/         → package "vantadb-llamaindex" (hatchling)
  packages/
    langchain-vantadb/  → package "langchain-vantadb" (setuptools)
    llamaindex-vantadb/ → package "llamaindex-vantadb" (setuptools)

Después:
  integrations/
    langchain/          → package "vantadb-langchain" (hatchling)
    langchain-vantadb/  → package "langchain-vantadb" (setuptools) [MOVED from packages/]
    llamaindex/         → package "vantadb-llamaindex" (hatchling)
    llamaindex-vantadb/ → package "llamaindex-vantadb" (setuptools) [MOVED from packages/]
  packages/             → [ELIMINAR después de validación]
```

### Estrategia B: Dejar `packages/` como está, archivar `integrations/`

Si se decide que los packages setuptools (`packages/`) son los "oficiales" y los hatchling (`integrations/`) son experimentales.

**Decisión requerida del usuario antes de implementar.**

### Archivos afectados (Estrategia A)

#### Lo que se mueve:
- `packages/langchain-vantadb/` → `integrations/langchain-vantadb/`
- `packages/llamaindex-vantadb/` → `integrations/llamaindex-vantadb/`

#### Lo que se actualiza:

| Archivo | Línea(s) | Cambio |
|---------|----------|--------|
| `.github/workflows/rust_ci.yml:19` | `- 'integrations/**'` | **Sin cambio** (trigger ya existe; packages no era trigger) |
| `.github/workflows/rust_ci.yml:36` | `- 'integrations/**'` | **Sin cambio** (idem) |
| `.github/workflows/rust_ci.yml:222` | `--ignore-filename-regex '...packages/experimental...'` | Cambiar `packages/experimental` → `integrations/experimental` (si existe) |
| `.github/workflows/rust_ci.yml:226` | `--ignore-filename-regex '...packages/experimental'` | Idem |
| `.opencode/AGENTS.md` | mención de `packages/` | Actualizar ruta |
| `docs/progreso/README.md` | mención de `TSK-139` | Confirmar que TSK-139 ya está resuelto |
| `docs/reviews/analisis_proyecto.md` | mención de `packages/` | Actualizar ruta en el análisis |
| `docs/CHANGELOG.md` | mención de `integrations/` | Posible update de paths |
| `.gitignore` | posible entrada para `packages/` | Remover si existe |

#### Lo que NO cambia:
- Los 8 crates `vantadb-*` (Rust PyO3) — son independientes, no se tocan
- Cargo.toml — no reference a `packages/` ni `integrations/`
- README.md — si menciona paths, actualizar
- `packages/experimental/` — si existe, mover también

#### Lo que se verifica después:
1. `git mv` preserva history
2. CI `rust_ci.yml` triggers siguen funcionando (push a `integrations/**`)
3. Los 4 packages se pueden instalar desde nueva ruta
4. `cargo build` no se ve afectado (no depende de estas rutas)
5. Symlinks temporales en `packages/` apuntan a `integrations/` para evitar 404

---

## M2: Unificar Benchmarks

**Problema:** Benchmarks en 3 ubicaciones (aunque `vantadb-python/benchmarks/` no existe, quedan 2):
- `benches/` — Rust benchmarks (6 archivos Criterion)
- `benchmarks/` — Python benchmarks (7 archivos + datos)

**Diagnóstico:** **No hay overlap real.** Los Rust benches miden microbenchmarks internos. Los Python benches miden stacks end-to-end. Mantener separados es correcto.

**Sin embargo:** El nombre confunde. `benchmarks/` sugiere benchmarks de alto nivel (son Python), y `benches/` sugiere benches de bajo nivel (son Rust). Está bien, es la convención de Cargo.

### Decisión: NO unificar, solo renombrar para claridad

| Ruta actual | Ruta nueva | Razón |
|---|---|---|
| `benchmarks/` → `benchmarks-python/` | ❌ Rechazado | Rompe CI, docs, scripts |
| `benchmarks/` → queda igual | ✅ **Elegido** | Es la convención, no hay overlap |

**Acción real:** No mover nada. Solo documentar explícitamente la separación en `docs/operations/BENCHMARKS.md`.

### Archivos afectados:
- `docs/operations/BENCHMARKS.md` — Agregar sección "Estructura de Benchmarks" explicando `benches/` (Rust) vs `benchmarks/` (Python)
- `.github/workflows/nightly_bench.yml` — Sin cambio
- `.github/workflows/bench.yml` — Sin cambio

### Riesgo: Ninguno. Es solo documentación.

---

## M3: Mover Artifacts de Raíz

**Problema:** 6 archivos huérfanos en la raíz del repo:
- `vanta_certification.json` — output de tests, ignorado por git
- `.vanta_profile` — profile de hardware generado, ignorado por git
- `architecture.png`, `engine.png`, `engine-mobile.png`, `homepage.png` — imágenes huérfanas

### M3.1: `vanta_certification.json`

**Referencias:**
- `tests/common/mod.rs:221` — `VantaHarness::REPORT_FILE = "vanta_certification.json"`
- `.gitignore:26` — ignorado

**Plan:**

1. Crear `dev-tools/reports/` si no existe
2. Cambiar `VantaHarness::REPORT_FILE` a `"dev-tools/reports/vanta_certification.json"`
3. Actualizar `.gitignore` — cambiar `/vanta_certification.json` → `dev-tools/reports/vanta_certification.json`
4. Si el archivo existe localmente, moverlo
5. **No tocar CI** — los tests escriben el archivo, no lo leen desde una ruta fija

**⚠️ Riesgo:** `tests/common/mod.rs` se compila en `cargo test`. Si el path cambia pero el directorio `dev-tools/reports/` no existe, el test falla al escribir. **Solución:** El test debe crear el directorio si no existe, o usar `std::fs::create_dir_all()`.

### M3.2: `.vanta_profile`

**Referencias:**
- `src/hardware/mod.rs:79` — `HardwareScout::PROFILE_PATH = ".vanta_profile"`
- `dev-tools/scripts/collect_code.ps1:73` — lo ignora (sin cambio)
- `docs/reviews/analisis_proyecto.md:1014` — mención documental
- `.gitignore:62` — ignorado

**Plan:**

1. Cambiar `HardwareScout::PROFILE_PATH` a `"dev-tools/reports/.vanta_profile"` o `"target/.vanta_profile"`
2. NO mover a `.cargo/` porque `.cargo/` no es un directorio estándar del proyecto
3. Opción recomendada: `"target/.vanta_profile"` (target/ ya existe, es temporal)
4. Actualizar `.gitignore` — cambiar `/.vanta_profile` → `target/.vanta_profile`
5. `collect_code.ps1` — ya ignora `.vanta_profile`, actualizar ruta si es necesario

**⚠️ Riesgo:** `HardwareScout` se usa temprano en el startup. Si el archivo no existe, `detect()` regenera el profile. Es seguro — el path es solo para cachear.

### M3.3: `architecture.png`, `engine.png`, `engine-mobile.png`, `homepage.png`

**Referencias:**
- **Cero referencias en código.** Solo aparecen en `.gitignore`.
- Son imágenes huérfanas, posiblemente mockups de diseño.

**Plan:**

1. Mover a `archive/root-artifacts/` (preservar history con `git mv`)
2. Si son capturas de diseño, subir a issues de diseño correspondiente
3. Actualizar `.gitignore` — cambiar paths de raíz a `archive/root-artifacts/`
4. No impacta nada más

**Riesgo:** Cero. Nadie las referencia.

### M3.4: `vantadb.rb` (raíz)

**Referencias:**
- Es una versión obsoleta de `Formula/vantadb.rb`
- No referenciado por CI, scripts, ni docs

**Plan:**

1. Mover a `archive/root-artifacts/vantadb.rb` (preservar history)
2. Confirmar que `Formula/vantadb.rb` es la oficial (sí, se confirmó)

**Riesgo:** Cero.

---

## M4: Subestructurar `docs/operations/`

**Problema:** 23 archivos sueltos en un solo directorio sin subestructura.

**Propuesta de subdirectorios:**

```
docs/operations/
├── README.md                    (NUEVO — índice del directorio)
├── BENCHMARKS.md                (queda — más referenciado)
├── CONFIGURATION.md             (queda — más referenciado)
├── PERFORMANCE_TUNING.md        (queda — más referenciado)
│
├── governance/
│   ├── AGENT_INSTRUCTIONS.md
│   ├── CI_POLICY.md
│   ├── COMMUNITY_GOVERNANCE.md
│   ├── EXPERIMENTAL_FEATURES.md
│   ├── REPO_CHECKLIST.md
│   └── SHOW_HN_PREP.md
│
├── release/
│   ├── PYTHON_RELEASE_POLICY.md
│   └── PUBLISHING_CHECKLIST.md  (NUEVO si aplica)
│
├── reliability/
│   ├── BACKUP_POLICY.md
│   ├── DURABILITY_GUARANTEES.md
│   ├── GC_TTL.md
│   ├── MEMORY_TELEMETRY.md
│   ├── RELIABILITY_GATE.md
│   └── SECURITY.md
│
├── monitoring/
│   ├── GRAFANA_SETUP.md
│   ├── grafana-dashboard.json
│   └── TELEMETRY.md (o renombrar desde MEMORY_TELEMETRY.md)
│
├── audit/
│   ├── EXECUTIVE_TECHNICAL_AUDIT.md
│   └── PUBLIC_ISSUE_DRAFTS.md
│
├── dev/
│   ├── EDITOR_INTEGRATIONS.md
│   └── FUZZING.md
│
└── snapshots/                   (ya existe para collect_code.ps1 output)
```

### Archivos que se MUEVEN (15 archivos):

| Archivo | Destino |
|---------|---------|
| `AGENT_INSTRUCTIONS.md` | `governance/` |
| `CI_POLICY.md` | `governance/` |
| `COMMUNITY_GOVERNANCE.md` | `governance/` |
| `EXPERIMENTAL_FEATURES.md` | `governance/` |
| `REPO_CHECKLIST.md` | `governance/` |
| `SHOW_HN_PREP.md` | `governance/` |
| `BACKUP_POLICY.md` | `reliability/` |
| `DURABILITY_GUARANTEES.md` | `reliability/` |
| `GC_TTL.md` | `reliability/` |
| `MEMORY_TELEMETRY.md` | `monitoring/` |
| `RELIABILITY_GATE.md` | `reliability/` |
| `SECURITY.md` | `reliability/` |
| `GRAFANA_SETUP.md` | `monitoring/` |
| `grafana-dashboard.json` | `monitoring/` |
| `EXECUTIVE_TECHNICAL_AUDIT.md` | `audit/` |
| `PUBLIC_ISSUE_DRAFTS.md` | `audit/` |
| `EDITOR_INTEGRATIONS.md` | `dev/` |
| `FUZZING.md` | `dev/` |
| `PYTHON_RELEASE_POLICY.md` | `release/` |
| `PILOT_PROGRAM.md` | `governance/` |

### Archivos que QUEDAN en `docs/operations/` (3 archivos):
- `BENCHMARKS.md` — más referenciado externamente (README, web, update_markdown.py)
- `CONFIGURATION.md` — más referenciado externamente
- `PERFORMANCE_TUNING.md` — más referenciado externamente

### Archivos que se ACTUALIZAN (referencias):

| Archivo | Referencia actual | Nueva referencia |
|---------|------------------|------------------|
| `README.md` (7 refs) | `docs/operations/X.md` | `docs/operations/category/X.md` |
| `README_ES.md` (7 refs) | `docs/operations/X.md` | `docs/operations/category/X.md` |
| `docs/FAQ.md` (2 refs) | `docs/operations/SECURITY.md`, `docs/operations/GC_TTL.md` | `docs/operations/reliability/SECURITY.md`, `docs/operations/reliability/GC_TTL.md` |
| `docs/CHANGELOG.md` (3 refs) | `docs/operations/X.md` | `docs/operations/category/X.md` |
| `docs/archive/REPORTE_INVESTIGACION_Y_DECISIONES.md` (1 ref) | `docs/operations/PILOT_PROGRAM.md` | `docs/operations/governance/PILOT_PROGRAM.md` |
| `web/src/routes/product/benchmarks.lazy.tsx` (1 ref) | `docs/operations/BENCHMARKS.md` | **Sin cambio** (queda en misma ruta) |
| `benchmarks/update_markdown.py` (1 ref) | `docs/operations/BENCHMARKS.md` | **Sin cambio** |
| `.github/CONTRIBUTING.md` (1 ref) | `docs/operations/CI_POLICY.md` | `docs/operations/governance/CI_POLICY.md` |
| `docs/architecture/STORAGE_VERSIONING.md` (1 ref) | `docs/operations/BACKUP_POLICY.md` | `docs/operations/reliability/BACKUP_POLICY.md` |
| `docs/progreso/README.md` (5+ refs) | `docs/operations/X.md` | `docs/operations/category/X.md` |
| `docs/bitacora.md` (1 ref) | `docs/operations/X.md` | `docs/operations/category/X.md` |
| `dev-tools/scripts/collect_code.ps1` (1 ref) | `docs/operations/snapshots/` | **Sin cambio** (snapshots/ no se mueve) |
| `.opencode/AGENTS.md` | directiva Doc-Driven | Actualizar paths |

### Total referencias a actualizar: ~32 referencias en ~13 archivos

### Estrategia de migración:

1. **Crear subdirectorios** con `git mkdir` (archivo .gitkeep en cada uno)
2. **Mover archivos** con `git mv` (preserva history)
3. **Actualizar referencias internas** entre los docs movidos (wikilinks)
4. **Actualizar referencias externas** en README, web, docs, CI
5. **Crear redirects** — archivos `.md` en las ubicaciones antiguas que digan:
   ```markdown
   > Este documento se ha movido a `docs/operations/governance/CI_POLICY.md`.
   > [Ir al nuevo destino](governance/CI_POLICY.md)
   ```
6. **Verificar** que `collect_code.ps1` sigue escribiendo a `snapshots/`
7. **Commit por subdirectorio** (commits atómicos)

### ⚠️ Riesgos específicos:

1. **Wikilinks internos rotos** — Los archivos `.md` en `docs/operations/` se referencian entre sí con wikilinks relativos (`[[CI_POLICY]]`). Moverlos rompe esos links.
2. **README.md y README_ES.md** — 14 referencias total. Error aquí rompe la documentación pública.
3. **`dev-tools/scripts/collect_code.ps1`** — escribe a `docs/operations/snapshots/`. Si `snapshots/` se mueve, se rompe.

---

## M5: Archivar `docs/reviews/`

**Problema:** 14 archivos de reportes de agente (one-shot) + 2 directorios vacíos. Solo 1 referencia externa.

### Archivos a archivar:

| Archivo | Destino |
|---------|---------|
| `docs/reviews/` directorio completo | `docs/archive/reviews/` |

**Excepción:** `docs/reviews/analisis_proyecto.md` tiene valor duradero (referenciado desde `docs/backlog-guide.md`).

**Dos opciones:**

**Opción A (Recomendada): Archivar todo**
- `git mv docs/reviews/ docs/archive/reviews/`
- Crear stub en `docs/reviews/README.md`:
  ```markdown
  > Este directorio se ha archivado en `docs/archive/reviews/`.
  > [Ver archivo](archive/reviews/)
  ```
- Actualizar `docs/backlog-guide.md:347` para apuntar a `docs/archive/reviews/analisis_proyecto.md`

**Opción B: Conservar `analisis_proyecto.md` en su lugar**
- Mover los 13 reportes de agente a `docs/archive/reviews/agents/`
- Dejar `analisis_proyecto.md` y `00-inventory/`, `14-redundant/`
- Actualizar `docs/backlog-guide.md`

### Archivos afectados:

| Archivo | Cambio |
|---------|--------|
| `docs/backlog-guide.md:347` | `docs/reviews/analisis_proyecto.md` → `docs/archive/reviews/analisis_proyecto.md` |

### Riesgo: Mínimo. Solo 1 referencia externa. Los reportes de agente no son consumidos por CI ni web.

---

## Orden de Ejecución Recomendado

```
Día 1: M1 (Consolidar integraciones) — requiere decisión del usuario
Día 2: M3 (Root artifacts) — bajo riesgo
Día 3: M4 (docs/operations/) — riesgo medio, muchas referencias
Día 4: M5 (docs/reviews/) — riesgo mínimo, 1 referencia
```

**M2 (Benchmarks)** es solo documentación — se puede hacer en paralelo con cualquier otra.

---

## Resumen de Riesgos por Migración

| Migración | Riesgo | Archivos movidos | Archivos editados | Referencias actualizadas |
|-----------|--------|-----------------|-------------------|-------------------------|
| M1 | Medio | ~2 dirs | 5-7 | 6-8 |
| M2 | Ninguno | 0 | 1 | 0 |
| M3 | Bajo | 6+ archivos | 4-5 | 3-4 |
| M4 | Medio-Alto | 19 archivos | 13+ | ~32 |
| M5 | Mínimo | 14 archivos | 2 | 1 |

---

## Rollback Plan

Para cada migración:

1. **Commits atómicos** por archivo/grupo — no mezclar migraciones en un commit
2. **Symlinks/stubs** en ubicaciones antiguas redirigen a nuevas ubicaciones por 2 semanas
3. **Si algo se rompe:** `git revert <commit>` restaura el estado anterior
4. **CI debe pasar antes del siguiente commit** — si `rust_ci.yml` falla después de M1, revertir inmediatamente
5. **Para M4 en particular:** Después de mover cada subdirectorio, correr `grep -r "docs/operations/\[old-path\]"` para detectar referencias huérfanas

---

## Checklist de Verificación Post-Migración

- [ ] `cargo build` compila sin errores
- [ ] `cargo test` pasa todos los tests
- [ ] `cargo clippy` sin nuevos warnings
- [ ] CI dry-run: simular un push a cada trigger path afectado
- [ ] `npm run build` en web/ compila
- [ ] Todos los READMEs referencian rutas correctas
- [ ] Los stubs/redirects funcionan (probarlos manualmente)
- [ ] `git log --follow <archivo_movido>` muestra history preservado
- [ ] No hay archivos huérfanos en la raíz (excepto Cargo.toml, README, LICENSE, .gitignore, etc.)
