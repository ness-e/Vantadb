# FND-03: Feature set mínimo + wheels CI compile matrix verde

## Metadata
- **Plan file:** docs/plans/2026-08-16-wave-p20-tsys.md
- **Fuente:** docs/Backlog.md:485 (P20a)
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡
- **Tipo:** Mixto (python, devops)
- **Turns estimados:** 15
- **Creado:** 2026-08-16T10:30
- **last-synced:** 2026-08-16T11:15
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | release-wheels-60.yml (job build-wheels), release.yml (release-plz), dev-tools/verify.ps1, AGENTS.md ritual de sesión (`cargo check --no-default-features --features fjall`) |
| Callees | vantadb-python (Cargo.toml dep del core con `default-features = false`), src/ (lib core, cfg-gated por features) |
| Implicaciones | Contrato público: feature set mínimo del core NO cambia; wheels empaquetan el set mínimo `fjall+memmap2+rayon`; no se toca semver |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `Cargo.toml` (workspace, 649L), `vantadb-python/Cargo.toml` (27L), `vantadb-python/pyproject.toml` (53L), `.github/workflows/release-wheels-60.yml` (295L), `docs/Investigaciones/FND-16-multi-target-ci.md` (129L)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `vantadb-python/Cargo.toml:24` → `vantadb = { path = "../", default-features = false, features = ["fjall", "memmap2", "rayon"] }`; `release-wheels-60.yml:80` → `cargo test --test version_coherence --no-default-features --features fjall`
- **Archivos que referencian a los editados (referencias entrantes):** grep FND-03 → Backlog.md:485, plan 2026-08-16-wave-p20-tsys.md:66, FND-23 (complementa decisión default-on/opt-in de grafos). No se editó ningún archivo existente.
- **Veredicto impacto:** bajo — verificación + reporte, cero cambios a código/CI

## Contrato
"`cargo check -p vantadb --no-default-features` pasa; reporte documenta features por target + abi3/manylinux declarado (o gap anotado); si hubo cambios de CI, actionlint pasa (`release-wheels-60.yml` si se tocó)."

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) el feature set mínimo del core (`--no-default-features`) debe seguir compilando — no introducir dependencias obligatorias no-feature; (2) el set que empaqueta el wheel es `fjall+memmap2+rayon` con `default-features = false` — no agregar extras al wheel; (3) abi3-py311 + manylinux 2_28 + musllinux 1_2 quedan como están.
- **Comandos de verificación:** `cargo check -p vantadb --no-default-features` ✅ (7 warnings pre-existentes, 0 errores); `cargo check -p vantadb --no-default-features --features fjall,memmap2,rayon` ✅ (0 errores)
- **Deuda pendiente:** ninguna (estado OK, sin gap)

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | Valor |
|------------------------|-------|
| `activeGoal` | FND-03: Feature set mínimo + wheels CI compile matrix verde |
| `lastAction` | DISCOVERY completo (Cargo.toml workspace, vantadb-python Cargo/pyproject, release-wheels-60.yml, FND-16) + verificación mecánica del contrato (2 checks cargo) + reporte `docs/Investigaciones/FND-03-wheels-feature-set.md` |
| `result` | OK (✅ COMPLETED) |
| `nextAction` | ninguno — tarea verificada sin gap, sin cambios |
| `contract` | verificacion: `cargo check -p vantadb --no-default-features` ✅ + `cargo check -p vantadb --no-default-features --features fjall,memmap2,rayon` ✅; evidencia: reporte FND-03 (features por target, abi3-py311, manylinux 2_28/musllinux 1_2); artefactos: docs/Investigaciones/FND-03-wheels-feature-set.md, task file FND-03.md; invariantes: feature set mínimo compila, wheel empaqueta fjall+memmap2+rayon; deuda: ninguna; queda_pendiente: FND-23 (default grafos con telemetría, post-launch) |
| `nextTask` | FND-23 (vanta-arch) |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda (no se introdujo código nuevo; solo verificación + reporte)

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate | Estado |
|-------|------|--------|
| **Task** | Contrato verificable se cumple (checks cargo ✅) + capa determinista (fmt/clippy/nextest no aplican — sin cambios de código) | ✅ |
| **Commit** | Sin commit — el lead commitea (instrucción explícita de la tarea) | ⏸️ |
| **Release** | verify.ps1 completo no aplica — sin cambios de código/CI | ⏸️ |

## Herramientas necesarias
- Terminal (cargo check), MetaSearchMCP (validación docs maturin), codegraph_explore (no aplicó — archivos de config)

## Investigation Notes
- **maturin `[tool.maturin] features`** (validado contra maturin.rs/config): activa features del crate construido vía cargo `--features`. `features = ["pyo3/extension-module"]` en pyproject.toml es válido (sintaxis `dep/feature`) y **redundante** — `extension-module` ya está en `vantadb-python/Cargo.toml:15`. No es un gap.
- **Config de wheels ya correcta:** `vantadb-python/Cargo.toml:24` fija el set mínimo del core (`default-features = false, features = ["fjall", "memmap2", "rayon"]`). El job maturin (`release-wheels-60.yml:82-90`) no pasa `--features` extra → empaqueta exactamente ese set mínimo. Nada que corregir.
- **FND-16 (ya commiteado):** los wheels ya corren por PR a main (3 OS + smoke tests). FND-03 complementa confirmando que el set empaquetado es el mínimo.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — NO aplica: no se tocan trust boundaries, no se agregan/quitan dependencias, no hay input de usuario. Solo verificación de config existente.
- [ ] **PERFORMANCE** — NO aplica: no se toca ningún hot path (sin código).

## Steps

### Step 1: DISCOVERY — Cargo.toml (workspace + vantadb-python), workflow wheels
- **Archivos:** `Cargo.toml`, `vantadb-python/Cargo.toml`, `vantadb-python/pyproject.toml`, `.github/workflows/release-wheels-60.yml`
- **Acción:** Leer config completa. Resultado: default features del core = `cli, arrow, fjall, roaring, advanced-tokenizer, memmap2, fs2, sysinfo, rayon`; vantadb-python depende del core con `default-features = false, features = ["fjall", "memmap2", "rayon"]`; pyo3 con `abi3-py311`; maturin action con `manylinux: 2_28`, `musllinux: 1_2`, sin `--features` extra; matrix 3 OS.
- **Verify:** lectura completa de los 4 archivos
- **Estado:** ✅

### Step 2: Verificación mecánica — feature set mínimo compila
- **Archivos:** ninguno (solo comandos)
- **Acción:** `cargo check -p vantadb --no-default-features` (contrato) + `cargo check -p vantadb --no-default-features --features fjall,memmap2,rayon` (set real del wheel).
- **Verify:** ambos exit code 0
- **Estado:** ✅

### Step 3: Implementación — solo si hay gap
- **Archivos:** `Cargo.toml`, `vantadb-python/Cargo.toml`, `vantadb-python/pyproject.toml`, `.github/workflows/release-wheels-60.yml`
- **Acción:** Análisis de gap: el set mínimo YA es lo que se empaqueta (dep del core en vantadb-python fija `default-features = false`), abi3 ya declarado, manylinux 2_28 ya declarado, matrix 3 OS ya corre en PR (FND-16). **Sin gap → sin cambios** (regla de la tarea: "Si ya está verde, documentá el estado como OK, no inventes cambios").
- **Verify:** no aplica (no hubo cambios → actionlint no requerido)
- **Estado:** ✅

### Step 4: Reporte — docs/Investigaciones/FND-03-wheels-feature-set.md
- **Archivos:** `docs/Investigaciones/FND-03-wheels-feature-set.md`
- **Acción:** Escribir reporte: features por target, verificación, estado OK, abi3/manylinux declarado, relación con FND-16.
- **Verify:** lectura del reporte generado
- **Estado:** ✅

## Dependencias
- FND-16 (ya commiteado): análisis multi-target CI — provee el contexto de que los wheels ya corren en PR.

## Review (GATE — agente distinto, P2-01)

> Tarea de verificación/documentación sin cambios de código. Review por agente distinto no requerido formalmente (no hay diff de código que revisar); la evidencia es mecánica (2 checks cargo ejecutados, salidas capturadas).

- **Revisor:** N/A (sin cambios de código; evidencia mecánica)
- **Enfoque:** N/A
- **Cómo se probó:** salidas reales de `cargo check` (0 errores) + validación docs oficiales maturin
- **Checklist anti-hábitos tóxicos:** N/A (sin implementación)
- **Veredicto:** ✅ approve

## Notas
- Los 7 warnings de `cargo check --no-default-features` son pre-existentes en `src/storage/vfile_mmap.rs` (`unnecessary unsafe block` con memmap2 0.9). No son de esta tarea ni bloquean el contrato.
- Backlog original de FND-03 mencionaba "aislamiento de features vector-only sin grafos". Alcance P20a (Backlog.md:485 DoD) = feature set mínimo compila + job CI verde — eso es lo que se verificó. El aislamiento fino de grafos queda delegado a FND-23 (decisión default-on/opt-in con telemetría post-launch).