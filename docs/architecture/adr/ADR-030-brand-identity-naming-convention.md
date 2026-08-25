---
title: "ADR-030: Identidad de marca — convención de nombres en artefactos públicos"
type: adr
status: proposed
tags: [vantadb, architecture, adr, branding, naming]
created: 2026-08-25
last_reviewed: 2026-08-25
---

# ADR-030: Identidad de marca — convención de nombres en artefactos públicos

> **PROPUESTA — pendiente decisión del owner (Regla 5).** La IA aporta la
> evidencia del mapeo (tabla abajo) y una convención propuesta lista para
> confirmar; el autor humano articula y confirma la decisión final. Hasta que
> el owner no la acepte (status → `accepted`), la convención es una propuesta.

## Context

Pre-launch, los artefactos públicos de VantaDB usan nombres distintos por
ecosistema sin una convención documentada. Auditoría realizada el 2026-08-25
con verificación live de cada registry:

| # | Artefacto | Nombre actual | Registry (verificado live 2026-08-25) | Repo URL | Homepage |
|---|---|---|---|---|---|
| 1 | Producto (display) | **VantaDB** | — | — | — |
| 2 | Repo GitHub | `ness-e/Vantadb` | ✅ público, 2⭐, 1381 commits | — | About = `vantadb.vercel.app` |
| 3 | Rust crate core | `vantadb` | ✅ crates.io 0.5.0 | `ness-e/Vantadb` | `vantadb.dev` ❌ dead |
| 4 | Rust CLI binary | `vanta-cli` | — (bin del crate) | — | — |
| 5 | Crates experimentales | `vantadb-server`, `vantadb-mcp`, `vantadb-wasm` | no publicados a crates.io (`publish=false` excepto core) | `ness-e/Vantadb` (wasm) | `vantadb.dev` ❌ dead (wasm) |
| 6 | Crates internos | `vanta-memory`, `vanta-proxy` | no publicados (`publish=false`) | — | — |
| 7 | PyPI distribución | `vantadb-py` | ✅ PyPI 0.5.0 (owner `DevpNess`) | `ness-e/Vantadb` | GitHub |
| 8 | Python módulo/import | `vantadb_py` (módulo), `vantadb` (import canónico según README local) | — | — | — |
| 9 | npm TS SDK | `vantadb` | ✅ npm 0.5.0 | `ness-e/Vantadb` | `vantadb.dev` ❌ dead |
| 10 | npm native bindings | `vantadb-node` | ❌ 404 — **nunca publicado en npm** | ausente en package.json | — |
| 11 | Dominio | `vantadb.dev` | ❌ DNS no resuelve (dominio comprado sin configurar) | — | — |

**Inconsistencias detectadas:**

1. **Homepage dispar:** los packages (crate, npm, wasm) declaran `homepage = https://vantadb.dev` — que NO resuelve (DNS muerto) — mientras el About del repo GitHub apunta a `vantadb.vercel.app` (live).
2. **`vantadb-node` nunca publicado** en npm: el `package.json` existe y tiene versión 0.5.0, pero `registry.npmjs.org/vantadb-node` devuelve 404.
3. **Identidad de cuenta dispar entre registries:** PyPI owner = `DevpNess`, crates.io owner = `ness-e`, GitHub = `ness-e` (misma persona, cuentas distintas).
4. **Metadata PyPI stale:** summary "Source-installed Python bindings…" y description en español (README viejo empaquetado en el release 0.5.0 de PyPI).
5. **Case del repo:** `ness-e/Vantadb` vs producto `VantaDB`. GitHub es case-insensitive en routing (los links funcionan), por lo que es cosmético — pero la URL canónica publicada en 60+ lugares usa `Vantadb`.
6. **Branch drift del README:** local (develop) documenta import canónico `vantadb`; el README publicado en GitHub (main) muestra `import vantadb_py` — drift entre ramas, no una decisión de naming.

**Verificación de links:** NO hay badges rotos — todos los workflows referenciados en README (`ci-rust-10.yml`, `gate-docs-21.yml`, `sec-codeql-30.yml`, `heavy-certification-50.yml`) existen en `.github/workflows/`. La única URL muerta es `vantadb.dev` (metadata de packaging + `enterprise@vantadb.dev` en SUPPORT.md), cuya resolución depende de la decisión de dominio del owner.

## Decisión (PROPUESTA — owner confirma)

**Convención de identidad de marca — sin renames:**

| Superficie | Nombre canónico | Uso |
|---|---|---|
| Producto | **VantaDB** | Display name en docs, badges, comunicaciones |
| Repo GitHub | **`ness-e/Vantadb`** | URL canónica publicada (se mantiene el case actual; rename a `VantaDB` sería cosmético y opcional — GitHub redirects) |
| Crate Rust | **`vantadb`** | `[dependencies] vantadb`, docs.rs |
| CLI | **`vanta-cli`** | Binario instalado por `cargo install`/scripts |
| PyPI | **`vantadb-py`** | `pip install vantadb-py`; import canónico `vantadb`, módulo `vantadb_py` (alias no roto) |
| npm (TS/WASM) | **`vantadb`** | `npm install vantadb` — SDK TypeScript WASM-powered |
| npm (nativo) | **`vantadb-node`** | `npm install vantadb-node` — bindings nativos (napi-rs); **pendiente publicar** |
| Dominio | **`vantadb.dev`** (intención) | DNS pendiente; mientras tanto GitHub About ya usa `vantadb.vercel.app` |

**Regla práctica:** el producto es VantaDB; cada ecosistema conserva el nombre
ya publicado (renames romperían semver/usuarios); la documentación nueva debe
usar los nombres de la tabla y nunca inventar variantes (ej. `vantadb-py` vs
`vantadb_py` vs `Vantadb`).

## Consecuencias

- **Pros:** cero breakage (no se renombra nada publicado); usuarios de cada
  ecosistema encuentran el paquete por el nombre que ya conocen; la
  documentación futura tiene una tabla canónica que impide variantes nuevas.
- **Cons / deuda asumida:** la divergencia de nombres entre ecosistemas
  persiste (es inherente a los registries: `vantadb` ya está tomado en npm,
  `vantadb-py` en PyPI); la homepage `vantadb.dev` sigue muerta hasta que el
  owner resuelva el DNS.

**Decisiones pendientes del owner (NO resueltas por este ADR):**

1. **Dominio canónico:** ¿configurar DNS de `vantadb.dev` (comprado) y unificar
   todos los homepages, o adoptar `vantadb.vercel.app` como web oficial? Hoy
   los packages apuntan a `vantadb.dev` (muerto) y GitHub About a `vantadb.vercel.app`.
2. **PyPI ownership:** transferir `vantadb-py` de `DevpNess` a `ness-e`
   (consistencia de cuenta entre registries).
3. **Publicar `vantadb-node`** en npm (existe el package.json; hoy no está publicado).
4. **(Opcional, cosmético)** rename del repo a `ness-e/VantaDB` para case
   consistente con el producto (GitHub redirige; cero riesgo de links rotos).
5. **Refrescar metadata PyPI** (summary + description en inglés del README actual).

## Alternativas consideradas

### Renombrar crates/packages para unificar (`vantadb` en todos lados)
- **Pros:** consistencia total de nombres.
- **Cons:** breaking semver en crates.io, npm y PyPI; migración forzada de
  usuarios; conflictos de disponibilidad (el nombre deseado puede estar
  tomado). **Rechazada:** STOP CONDITION del plan FIND-17 — los renames
  romperían semver/usuarios; se documenta, no se fuerza.

### Alias de paquete (`vantadb` como dist de PyPI)
- **Pros:** import idéntico entre ecosistemas.
- **Cons:** requiere registro nuevo + deprecar `vantadb-py`; PyPI exige que el
  nombre no colisione; complejidad de publicación. **Rechazada:** el import
  canónico `vantadb` ya cubre la experiencia del usuario; el dist name
  `vantadb-py` es solo metadata de instalación.