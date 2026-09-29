---
title: "ADR-0045: Naming freeze 0.7.0→1.0 — congelamiento de los 9 artefactos + política de alias"
kind: adr
status: accepted
description: "Con 0.7.0 ya publicado en los tres registries (crates.io vantadb,"
tags: [vantadb, architecture, adr, naming, semver, deprecation]
created: "2026-09-27"
---

# ADR-0045: Naming freeze 0.7.0→1.0 — congelamiento de los 9 artefactos + política de alias

> **ACEPTADO — firmado por el owner el 2026-09-27 (Regla 5).** El congelamiento
> de los 9 artefactos (D1) y la política de alias (D2) están en vigor; el ADR se
> complementa con ADR-030 (D3). Evidencia: tabla 9/9 verificada `file:line` +
> registry live (2026-09-27). Firma y articulación en [§ Owner sign-off](#owner-sign-off-regla-5).

## Context

Con `0.7.0` ya publicado en los tres registries (crates.io `vantadb`,
PyPI `vantadb-py` y npm `vantadb` sirven 0.7.0 — verificado live 2026-09-27) y la campaña de estandarización de
API (`API-01..09`) cerrada como último tramo breaking pre-lanzamiento, la ventana
para congelar nombres sin costo de migración se cierra: renombrar un artefacto ya
publicado cuesta migraciones, soporte y crédito del proyecto (R1 del master
roadmap).

`ADR-030` (status `proposed`) fijó la convención de identidad de marca y la regla
práctica **"no renames; cada ecosistema conserva el nombre ya publicado"**, pero
opera a nivel de marca (11 superficies + display + dominio) y deja 5 decisiones
del owner pendientes (dominio, PyPI ownership, publicar `vantadb-node`, case del
repo, metadata PyPI — carril LEG-01). Falta el congelamiento a nivel de
**artefacto** para el tramo `0.7.0 → 1.0` y la mecánica de alias/deprecación para
cualquier cambio de nombre futuro.

**Los 9 artefactos a congelar (evidencia file:line, 2026-09-27):**

| # | Nombre congelado | Qué nombra | Registry / estado (live 2026-09-27) | Evidencia |
|---|---|---|---|---|
| 1 | `vantadb` | Crate Rust core | crates.io **0.7.0** ✅ | `Cargo.toml:2` |
| 2 | `vantadb-py` | Distribución Python (PyPI) | PyPI **0.7.0** ✅ | `vantadb-python/pyproject.toml:6` |
| 3 | `vantadb-node` | Bindings nativos napi-rs | npm **404 — nunca publicado** | `vantadb-node/package.json:2-3` |
| 4 | `vantadb-ts` | Dir/repo del SDK TypeScript (npm publica como `vantadb`) | dir ✅ + npm `vantadb` **0.7.0** ✅ | `vantadb-ts/package.json:2` |
| 5 | `vantadb-server` | Crate HTTP server | workspace · `publish=false` | `vantadb-server/Cargo.toml:2,6` |
| 6 | `vantadb-mcp` | Crate MCP server | workspace · `publish=false` | `vantadb-mcp/Cargo.toml:2,7` |
| 7 | `vanta-cli` | Binario CLI | `[[bin]]` del crate core | `Cargo.toml:350-352` |
| 8 | `vanta-proxy` | Crate LLM proxy | workspace · `publish=false` | `vanta-proxy/Cargo.toml:2,6` |
| 9 | `vanta-memory` | Crate memoria L0–L3 | workspace · `publish=false` | `vanta-memory/Cargo.toml:2,7` |

La documentación de las 11 superficies públicas (`docs/api/VERSIONING.md`) debe
usar estas grafías para los artefactos listados; la divergencia entre ecosistemas
— `vantadb` en crates.io/npm vs `vantadb-py` en PyPI — es esperada y se conserva.
(`vantadb-wasm`, superficie 5, no integra la lista de 9 del contrato; la regla
"no renames" de `ADR-030` y la política D2 aplican igual a su nombre.)

**Precedente interno (contexto, no vinculante hacia atrás):** `ADR-041` (renames
del enum `Error`), `ADR-042` (firmas `pub`), `ADR-044` (acumulado breaking 0.6.0)
documentan la práctica pre-usuarios: rename directo, sin aliases, con
`feat!:`/`BREAKING CHANGE:` — válida con "0 usuarios". Ese régimen queda cerrado
para nombres de artefacto a partir de `0.7.0` con este ADR.

## Decision

> **[OWNER — Regla 5]** ✅ Confirmado por el owner el 2026-09-27 (firma en §Owner sign-off).

**D1 — Congelamiento 9/9 (`0.7.0 → 1.0`).** Los 9 nombres de la tabla quedan
congelados: ningún artefacto se renombra entre `0.7.0` y `1.0`. Cualquier rename
futuro requiere un ADR nuevo que enmiende o reemplace a este ADR (firmado por el
owner) — **nunca un rename silencioso**.

**D2 — Política de alias/deprecación.** Todo cambio de nombre público sigue esta
mecánica, sin excepciones:

1. **Alias funcional** — el nombre viejo sigue funcionando (shim, re-export o
   alias según ecosistema; el mecanismo concreto se elige al deprecar, el
   invariante es el mismo).
2. **Fecha de remoción obligatoria** — deprecado durante ≥1 MINOR antes de la
   remoción (`VERSIONING.md` §Deprecation policy); la remoción lleva
   `feat!:`/`BREAKING CHANGE:` y un target explícito.
3. **Registro** — entrada en `docs/api/DEPRECATIONS.md` (surface, deprecated-in,
   removal target, migración) + entrada de changelog.
4. **Post-1.0** — un cambio breaking de nombre solo en un **MAJOR** (nunca en un
   MINOR): semver estándar + la ventana de deprecación de arriba.
5. **Prohibido** — el rename silencioso (cambiar la grafía en docs/código sin
   alias + fecha + registro). Es el único invariante no negociable.

**D3 — Relación con ADR-030.** Complementario, no lo supersede: `ADR-030` gobierna
identidad de marca (display, homepage, cuentas, repo); `ADR-045` congela los
nombres de artefacto y define la mecánica de alias. Las 5 decisiones pendientes
de `ADR-030` se referencian sin resolver (carril owner / LEG-01).

## Alternatives Considered

### Renombrar artefactos para unificar en `vantadb`
- Pros: consistencia total de nombres.
- Cons: breaking semver en registries con usuarios; nombres tomados
  (`vantadb-py` ya existe en PyPI); migración forzada.
- **Rechazada:** costo post-usuarios >> beneficio cosmético (R1; ADR-030 ya la
  rechazó a nivel de marca).

### Rename directo sin alias (práctica pre-0.7, ADR-041/042/044)
- Pros: cero deuda de compatibilidad; la ventana 0.x lo permite.
- Cons: los usuarios de `0.7.0` rompen sin aviso; el contrato del master lo
  prohíbe ("nunca rename silencioso").
- **Rechazada:** válida sin usuarios; `0.7.0` ya está publicado.

### Alias sin fecha de remoción ("eternos")
- Pros: cero ruptura y cero fricción.
- Cons: deuda perpetua; contradice `DEPRECATIONS.md` (removal target obligatorio).
- **Rechazada:** F2 del pre-mortem del master — fecha obligatoria.

### Congelar recién en `1.0`
- Pros: deja margen de rename.
- Cons: `1.0` llega con más usuarios → mayor costo de migración.
- **Rechazada:** congelar pre-usuarios es gratis; después cuesta (R1).

## Consequences

- **Pros:** los consumidores pueden fijar `vantadb` / `vantadb-py` / … en
  dependencias, docs y scripts con estabilidad hasta `1.0`; adapters y anuncio
  (MKT-18f) publican contra nombres definitivos; cero costo de migración por
  renames.
- **Cons / deuda asumida:** la divergencia de grafía entre ecosistemas persiste
  (npm `vantadb` vs PyPI `vantadb-py`) — inherente a los registries; cualquier
  cambio futuro de nombre arrastra el costo de la mecánica de alias (alias +
  ventana + registro), por diseño.
- **Riesgo residual:** `vantadb-node` sigue sin publicar (404); el freeze fija su
  nombre **antes** de la primera publicación (barato: no hay consumidores aún).
  Publicarlo es deuda de owner/MKT, no de este ADR.
- **[OWNER — Regla 5]** ✅ Pros/contras/riesgos aceptados por el owner el 2026-09-27 (ver §Owner sign-off).

## Owner sign-off (Regla 5)

> **[OWNER]** Firmado vía Gate P (question tool, 2026-09-27): **"Firmo: apruebo D1–D3"**.
>
> - **Articulación:** el owner ratifica el congelamiento 9/9 (D1), la política de alias con fecha de remoción obligatoria (D2) y la relación complementaria con ADR-030 (D3) tal como quedaron articuladas en el draft de evidencia.
> - **Firma:** Eros (owner) — 2026-09-27
> - **Riesgos aceptados:** divergencia de grafía entre ecosistemas (npm `vantadb` vs PyPI `vantadb-py` — inherente a los registries); `vantadb-node` sin publicar (nombre congelado antes de la primera publicación; publicarlo es deuda owner/MKT).

## References

- Contrato origen: master roadmap Task 12 (`docs/dev/plans/2026-09-26-master-roadmap.md`) · Backlog `DEF-04`.
- `ADR-030` — convención de marca (11 superficies; 5 decisiones pendientes del owner).
- `docs/api/VERSIONING.md` — 11 superficies + §Deprecation policy (freeze + alias).
- `docs/api/DEPRECATIONS.md` — registro de instancias (HARD-01).
- Task file: `docs/dev/tasks/DEF-04.md` (evidence pack + verify).
