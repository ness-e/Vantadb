# DEPS-01 — Crates duplicadas en el grafo de dependencias

> **Fecha:** 2026-08-06
> **Origen:** Sección 7 del Audit Report 2025-07-27, re-verificado contra Cargo.lock actual.
> **DoD:** Reporte con la tabla completa por crate. **Ninguna versión modificada en Cargo.lock** (este reporte es puramente investigativo — no se tocó el lockfile).

## Resumen ejecutivo

De las **8 crates** duplicadas señaladas en el audit original, el estado real hoy es:

- **2 ya consolidadas** desde el audit: `thiserror` (ahora solo `1.0`) y `lru` (solo `0.16`). El audit decía `thiserror 1.x/2.x` y `lru 0.12/0.16` → quedan asurano.1.
- **6 siguen multi-versión**: `hashbrown`, `rand`, `rand_core`, `getrandom`, `reqwest`, `windows-sys`. Todas son **duplicación transitiva legítima con causas mayores de MSRV/API**, no consolidadas sin riesgo.

## Tabla por crate

| Crate | Versiones (lock actual) | Causa raíz (quién exige cada versión) | ¿Unificable? | Riesgo de unificar | Recomendación |
|---|---|---|---|---|---|
| `hashbrown` | 0.14.5, 0.16.1, 0.17.1 | 0.14.5 ← `dashmap 6.2` (→ fjall); 0.16.1 ← `lru 0.16` + `quick_cache` (→ lsm-tree/fjall); 0.17.1 ← `arrow` 59 | **No** | **Alto** — 3 majors; cada consumidor pinnea su mayor por hashbrown's `raw_entry`/layout breaking changes | Dejar multi-versión. Revisitar cuando arrow/dashmap/lsm-tree suban a un hashbrown común (≥0.17). |
| `rand` | 0.9.5, 0.10.2 | 0.9.5 ← directa `vantadb` + `proptest` (dev); 0.10.2 ← `twox-hash 2.1` (→ lz4_flex/fjall) | **No** | **Alto** — `rand 0.10` es mayor con breaking API; las dependencias que lo exigen son de fjall's stack | Mantener. La directa 0.9.5 es la que usa `vantadb` (SeedableRng no 0.10-proof). |
| `rand_core` | 0.9.5, 0.10.1 | 0.9.5 ← `rand 0.9` + `rand_chacha`; 0.10.1 ← `chacha20` → `rand 0.10` (two-a-hash stack) | **No** | **Alto** (idem rand) | Acompaña a rand; se resuelve si suben la cadena fjall. |
| `getrandom` | 0.2.17, 0.3.4, 0.4.3 | 0.3.4 ← `ahash→arrow`; 0.4.3 ← `jobserver→cc` (build de criterion); 0.2.17T restos de cadena rand 0.8 | **No** | **Alto** — 3 majors; 0.4 es la línea activa | Migrar cuando `ahash`/arrow y `cc` acepten 0.4. Mantener por ahora. |
| `reqwest` | 0.12.28 (crates|| linea direct) , 0.13.4 (solo con `--all-features`) | 0.12 ← directa `vantadb` (remote-inference/wal-shipping); 0.13 ← `opentelemetry-http 0.32` → `opentelemetry-otlp` | **Parcial** | **Medio** | La directa exige 0.12 (código depende de `blocking`, `json`). `opentelemetry-otlp 0.32` trae 0.13. Solo unificable subiendo la directa a 0.13 si el API del código es compatible — validar con `cargo semver-checks` antes. Dejar multi-versión (es solo features opt-in no normal). |
| `thiserror` | solo **1.0** (antes 1.x/2.x) | — | ✅ ya consolidada | — | **NINGÚN cambio** — ya single-version en lockfile actual. |
| `lru` | solo **0.16** (antes 0.12/0.16) | — | ✅ ya consolidada | — | **NINGÚN cambio** — ya single-version. |
| `windows-sys` | 0.59.0, 0.61.2 | 0.59 ← `fs4 0.13` (tantivy); 0.61 ← `anstyle-query→clap` tree | **No** | **Medio** — 2 minors con feature-set distinto | Multi-versión legítima (minor pinning de feature-set por crate). No unificar manualmente. |

## Análisis por categoría

1. **Majors legítimas por MSRV/API** (alto riesgo, no unificar sin subir dependecias): `hashbrown`, `rand`, `rand_core`, `getrandom`.
2. **Transitivas consolidables con bump de dependencia** (medio, opcional): `reqwest` — depende de subir la directa a 0.13 y de que `opentelemetry-otlp` coincida.
3. **Feature-gating por plataforma** (no accionable): `windows-sys`.
4. **Ya consolidadas** (`thiserror`, `lru`): el lockfile actual ya las resolvió a single version. **Acción tomada: ninguna** — el DoD pide no modificar versiones; quedó verificado.

## Recomendación final

Ninguna crate duplicada exige acción inmediata. Las 6 restantes son duplicación transitiva con causas legítimas de MSRV/API. La unificación real vendrá de **subir las dependencias anchas** (arrow, fjall/quick_cache/lru, ahash) a versiones que compartan una sola `hashbrown`/`rand`, no de forzar versión en Cargo.lock.

**Próximo paso (cuando haya "cargo update" mayor):** volver a correr `cargo tree -i` para las 6 crates; las que hayan convergido a una versión se marcan resueltas en backlog.

## Verificación

- `cargo tree -i <crate>@<version>` para las 6 crates restantes ejecutado 2026-08-06.
- **Ninguna versión modificada en Cargo.lock** ✅ (DoD).
- `cargo check --workspace` no ha cambiado (solo lectura — reporte).