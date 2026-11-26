---
title: "FIND-708: monitor de supply chain — chrono vía arrow y fecha del waiver"
kind: task
status: completed
description: "Estado medido del último residuo sin maintainer del grafo (chrono 0.4.45 tras arrow-arith 59.3.0): sin cambios al 2026-11-24, waiver renovado a 2027-03-22, sin ruta de salida upstream"
---

# FIND-708: monitor de supply chain — chrono vía arrow y fecha del waiver

## Metadata
- **Plan file:** [2026-11-23-release-v0.10.0-gates.md](../plans/2026-11-23-release-v0.10.0-gates.md) — §6, Fase 2, wave 2.3
- **Fuente:** fila `FIND-708` del plan (monitor, ~30 min/mes); hermano de `FIND-652` (migración directa chrono→jiff, cerrada)
- **Esfuerzo:** 🟢 30 min (monitor recurrente)
- **Prioridad:** 🟡 Media (riesgo aceptado, no remediable hoy)
- **Tipo:** Docs / supply chain (auditoría, sin cambios de dependencias)
- **Turns estimados:** 3
- **Creado:** 2026-11-24T03:40
- **last-synced:** 2026-11-24T03:55
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | ninguno de código — la tarea solo lee el grafo |
| Callees | `Cargo.toml` (`arrow` opcional, L87/L159), `Cargo.lock` (chrono 0.4.45, arrow 59.3.0), `deny.toml` (L9-21), `osv-scanner.toml` (L23-34), `docs/specs/CONSTRAINTS.md` (W3) |
| Implicaciones | Ninguna: **no se toca ninguna dependencia** (`cargo update`/`cargo add`/`Cargo.toml` prohibidos por el contrato). El único efecto es documental: este task file + la entrada de registro en `dependencias.md` |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `deny.toml` (L1-80), `osv-scanner.toml` (L1-45), `docs/dev/avance/auditoria/dependencias.md` (L1-101), `Cargo.toml` (features L152-173, targets L458-471, dep L87), `.opencode/task-system/prompts/task.md`
- **Archivos referenciados hacia dentro:** `src/lib.rs:89-90` (`#[cfg(feature = "arrow")] pub mod columnar;`), `src/columnar.rs` (182 L, un `pub fn nodes_to_record_batch`), `tests/logic/columnar.rs` (`required-features = ["arrow"]`)
- **Archivos que referencian a los editados (referencias entrantes):** `rg -n 'chrono|arrow' deny.toml .github/workflows/*.yml` → `deny.toml:9` es el único hit de advisory/waiver; ningún workflow menciona chrono (los 5 que dicen "arrow" lo hacen por features de build). `docs/specs/CONSTRAINTS.md:96` (W3) cita el mismo waiver
- **Veredicto impacto:** nulo sobre código y sobre el build. Solo se añaden dos documentos de auditoría; el waiver en `deny.toml`/`osv-scanner.toml` queda intacto

## Contrato

```
cargo tree -i chrono                       # default build: "did not match any packages" (sin chrono)
cargo tree -i chrono --features arrow      # cadena completa arrow-arith 59.3.0 -> arrow 59.3.0 -> chrono 0.4.45
cargo tree -e features -i chrono --features arrow   # chrono feature "clock" entra por arrow-arith feature "default"
cargo deny check advisories                # advisories ok (exit 0), RUSTSEC-2026-0543 advisory-not-detected
rg -n 'chrono|arrow' deny.toml .github/workflows/*.yml | rg -i 'advisory|waiver|ignore|RUSTSEC'   # 1 hit: deny.toml:9
```

Completado = las 5 salidas arriba quedan pegadas en `## Verificación` y la entrada de registro en
`docs/dev/avance/auditoria/dependencias.md` describe estado + fecha de waiver + decisión al vencer.

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (a) `chrono` NO es alcanzable en el build por defecto — si alguien lo vuelve default (o mete `arrow` en `default`), el waiver deja de ser "riesgo residual" y pasa a ser riesgo de release; (b) `deny.toml` y `osv-scanner.toml` deben declarar la MISMA fecha (2027-03-22) y el mismo owner; (c) ninguna remediación puede consistir en silenciar el advisory sin owner+expiry.
- **Comandos de verificación:** `cargo tree -i chrono` (debe seguir sin match) · `cargo tree -i chrono --features arrow` (debe seguir mostrando chrono 0.4.x) · `cargo deny check advisories` (exit 0)
- **Deuda pendiente:** el waiver sigue vigente y no hay ruta de salida upstream al 2026-11-24; queda la decisión del owner al vencer (2027-03-22 o antes si `arrow-arith` suelta chrono).

## Verificación (salidas reales, 2026-11-24)

### 1. `cargo tree -i chrono` (build por defecto)

```
error: package ID specification `chrono` did not match any packages

help: a package with a similar name exists: `camino`
```

Causa: `Cargo.toml:156` → `default = ["cli", "fjall", "memmap2", "fs2", "roaring"]`. **`arrow` no está en
default** (`arrow = ["dep:arrow"]`, L159), así que chrono no entra al grafo.

### 2. `cargo tree -i chrono --features arrow`

```
chrono v0.4.45
├── arrow-arith v59.3.0
│   └── arrow v59.3.0
│       └── vantadb v0.9.0 (C:\Users\Eros\VantaDB Proyect\VantaDB)
│           ├── vanta-memory v0.9.0
...
├── arrow-array v59.3.0
│   ├── arrow v59.3.0 (*)
│   ├── arrow-arith v59.3.0 (*)
│   ├── arrow-cast v59.3.0
...
└── arrow-cast v59.3.0 (*)
```

### 3. `cargo tree -e features -i chrono --features arrow` (por qué entra)

```
chrono v0.4.45
├── chrono feature "alloc"
│   └── chrono feature "std"
│       └── chrono feature "now"
│           └── chrono feature "clock"
│               ├── arrow-arith v59.3.0
│               │   └── arrow-arith feature "default"
│               │       └── arrow v59.3.0
│               │           └── vantadb v0.9.0
```

`arrow` declara **`arrow-arith` como dependencia normal no opcional** (verificado en
`arrow 59.3.0` vía crates.io API: `arrow-arith ^59.3.0 normal optional=False`), y `arrow-arith`
arrastra `chrono ^0.4.40` también **no opcional**. Es decir: **`default-features = false` en
`Cargo.toml:87` no protege de nada** — el consumidor que encienda `arrow` siempre paga chrono.

### 4. `cargo deny check advisories`

```
warning[advisory-not-detected]: advisory was not encountered
   ┌─ deny.toml:21:6
21 │     "RUSTSEC-2026-0543",
   │      ━━━━━━━━━━━━━━━━━ no crate matched advisory criteria
advisories ok
EXIT=0
```

El ignore **no se dispara en el grafo por defecto** (chrono no está), pero sí aplicaría en cualquier
build con `--features arrow`. Es un `warning`, no un error: gate verde, exit 0. (El otro warning,
`RUSTSEC-2023-0089` en `deny.toml:29`, es preexistente y de otro dominio.)

### 5. Contrato `rg`

```
deny.toml:9:    # RUSTSEC-2026-0543: chrono is unmaintained (informational; no patched version).
--- workflow chrono/arrow refs ---
.github/workflows/nightly.yml
.github/workflows/release-binaries.yml
.github/workflows/ci-rust.yml
.github/workflows/release.yml
.github/workflows/certification.yml
```

Ningún workflow menciona chrono ni el advisory: los 5 hits de `arrow` son features de build
(`--features arrow` en certificación/release). **Hay un solo punto de waiver, y está en `deny.toml`.**

## Estado medido (2026-11-24)

| Medida | Valor |
|---|---|
| `chrono` en build por defecto | **no** (`cargo tree -i chrono` → sin match) |
| `chrono` con `--features arrow` | **sí**: 0.4.45 (único en el lock) |
| Cadena | `vantadb → arrow 59.3.0 → arrow-arith 59.3.0 → chrono 0.4.45` |
| Advisory | **RUSTSEC-2026-0543** (INFO/Unmaintained, *all versions*, **sin versión parcheada**) |
| Fecha del advisory | reportado 2026-09-14 · emitido 2026-11-17 |
| Waiver en `deny.toml` | L9-21, owner `@ness-e` (vanta-lead), **review obligatoria 2027-03-22** (renovado 2026-11-24) |
| Waiver en `osv-scanner.toml` | L31-34, `ignoreUntil = "2027-03-22T00:00:00Z"` (mismo owner, renovado 2026-11-24) |
| Tercer espejo | `docs/specs/CONSTRAINTS.md:96` (excepción **W3**) |
| `cargo deny check advisories` | ✅ exit 0 (`advisory-not-detected`, warning) |
| Versión de `arrow` en el lock | 59.3.0 (última 59.x publicada: 59.3.3; última global: **60.2.0**, 2026-11-21) |

## Ruta de salida — ¿existe? (investigación 2026-11-24)

**No. Bumpear `arrow` no rompe la cadena.** Verificado contra el registro, no contra memoria:

| Crate / versión | ¿Depende de chrono? | Evidencia |
|---|---|---|
| `arrow-arith` 59.3.3 (última 59.x) | **sí**, `chrono ^0.4.40`, normal, **no opcional** | `crates.io/api/v1/crates/arrow-arith/59.3.3/dependencies` |
| `arrow-arith` 60.2.0 (última global) | **sí**, `chrono ^0.4.40`, normal, **no opcional** | `crates.io/api/v1/crates/arrow-arith/60.2.0/dependencies` |
| `chrono` (cualquier versión) | n/a — **no existe versión parcheada**; el crate está sin maintainer | [RUSTSEC-2026-0543](https://rustsec.org/advisories/RUSTSEC-2026-0543.html) |

Esto cierra la pregunta que quedó abierta en `osv-scanner.toml:29` ("¿arrow 60 suelta chrono?"):
**la respuesta es NO** al 2026-11-24. El advisory es de tipo *Unmaintained* (INFO), no de
vulnerabilidad: no hay CVE, no hay exploit, no hay fix posible — solo la decisión de mantener o no
la dependencia.

### Decisión que toca al owner al vencer (2027-03-22, o antes si cambia upstream)

| Opción | Qué implica | Coste estimado |
|---|---|---|
| **A. Retirar la feature `arrow`** (recomendada si `columnar` no tiene usuarios) | Borrar `src/columnar.rs` (182 L, 1 `pub fn nodes_to_record_batch`), `tests/logic/columnar.rs`, la fila de `src/lib.rs:30`, el gate `required-features = ["arrow"]` (Cargo.toml:466) y retirar el ignore de `deny.toml`+`osv-scanner.toml` (quedarían stale). `arrow` sale del grafo → chrono desaparece del todo | **~2 h** (código 30 min, docs/UPGRADE 30 min, verify 45 min). Es *breaking* de API pública → necesita entrada en CHANGELOG vía release-plz (`feat!:`/`BREAKING`) |
| **B. Asumir el riesgo documentado** (renovar el waiver) | No se toca nada; se re-firma owner+expiry. Justificable mientras `arrow` siga fuera de default: **el binario publicado no contiene chrono** | **~15 min/mes** de monitor. Riesgo: advisory INFO sin fix; el coste real es la deuda de "un crate sin maintainer en el grafo" |
| C. Reemplazar `arrow` por otro columnar | Reescribir `src/columnar.rs` sobre otra lib | **~1-2 d**, sin ganancia medible — descartada (YAGNI) |

**Nota de calibración:** la pérdida de `columnar` es real pero acotada — un único `pub fn` exportado
y su test. Si algún consumidor externo usa `vantadb::columnar::nodes_to_record_batch`, A rompe su
build; pre-1.0 eso es un minor. La decisión A vs B depende de si existe ese consumidor, dato que
el owner tiene y este monitor no puede medir localmente (`docs.rs` reverse deps / issues).

## Steps

### Step 1: medir el grafo (default + `--features arrow`)
- **Archivos:** ninguno (solo lectura)
- **Acción:** `cargo tree -i chrono` y `cargo tree -i chrono --features arrow` + `-e features`
- **Verify:** salidas pegadas en §Verificación 1-3
- **Estado:** ✅ COMPLETED

### Step 2: localizar el waiver y su disparador
- **Archivos:** `deny.toml`, `osv-scanner.toml`, `docs/specs/CONSTRAINTS.md`, `.github/workflows/*.yml`
- **Acción:** `rg 'chrono|arrow' … | rg -i 'advisory|waiver|ignore|RUSTSEC'` + `cargo deny check advisories`
- **Verify:** §Verificación 4-5
- **Estado:** ✅ COMPLETED

### Step 3: ¿existe ruta de salida? (registro upstream)
- **Archivos:** ninguno
- **Acción:** consultar `arrow-arith` 59.3.3 / 60.2.0 en crates.io + el advisory en rustsec.org
- **Verify:** tabla §Ruta de salida
- **Estado:** ✅ COMPLETED

### Step 4: registrar el estado
- **Archivos:** `docs/dev/tasks/FIND-708.md`, `docs/dev/avance/auditoria/dependencias.md`
- **Acción:** escribir este task file + la entrada `FIND-708` en el registro de auditoría
- **Verify:** `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs`
- **Estado:** ✅ COMPLETED

## Dependencias

- `FIND-652` (cerrada): migración de las dependencias **directas** chrono→jiff 0.2. Sin ella este waiver no sería "residuo".
- Ninguna dependencia de ejecución con otras tareas de la wave 2.3.

## Review (GATE — agente distinto, P2-01) — **degradado (marcado)**

- **Tier:** **Fast** — paths del diff: `docs/dev/tasks/FIND-708.md`, `docs/dev/avance/auditoria/dependencias.md`, `docs/index.md` (regenerado). Ninguno matchea los globs adversariales (`docs/api/**`, `src/**`, wire formats). Aplica verify fast mecánico + veredicto registrado.
- **Revisor:** sin sub-agente disponible en esta invocación → **`doubt-driven-development` degradado**, marcado como degradado, + escalado al owner (ver §Notas).
- **Enfoque:** ¿son correctas las tres afirmaciones centrales? (a) chrono no alcanzable por defecto; (b) arrow-arith 60.2.0 sigue exigiendo chrono; (c) fecha de waiver 2027-03-22 consistente en los tres espejos.
- **Cómo se probó:** no auto-reporte — cada claim tiene comando + salida pegada (§Verificación) o URL de registro (§Ruta de salida). El claim (b) es el único que dependía de conocimiento externo: se resolvió consultando la API de crates.io, no memoria.
- **Gates docs:** `node scripts/docs/check-docs.mjs` → **exit 0**. `node scripts/docs/check-links.mjs` → exit 1 con **5 links rotos en ficheros ajenos a este diff** (`docs/user/IS_IT_FOR_ME.md` ×4, `docs/user/WHICH_RUNTIME.md` ×1, todos apuntando a `concepts/what-is-vantadb.md` — trabajo en vuelo de `FIND-323`); **0 rotos en los archivos de FIND-708** (el único que introduje, `../tasks/FIND-708.md`, se corrigió a `../../tasks/FIND-708.md` antes del commit). `gen-index.mjs --write` **no se commitea**: su diff arrastra las entradas de `FIND-323` + `what-is-vantadb.md` de la wave paralela; se revirtió para no commitear trabajo de otro agente.

- **Checklist anti-hábitos tóxicos:** sin salidas inventadas (todas pegadas) · sin saltarse clarificación (no había decisión abierta que tomar: el contrato prohíbe tocar deps) · sin "done" sin verificar (5 comandos + 2 fetches) · sin ignorar fallos (`cargo tree -i chrono` "falla" y eso **es** el resultado, documentado) · sin bucle de reintentos · sin huérfanos · sin degradar chequeo (el waiver NO se tocó) · parada explícita · SDP cubierto (`documentation-skill` por escritura en `docs/`, `systematic-debugging` del lifecycle VERIFY declarado como no aplicable: no hay bug que diagnosticar).
- **Veredicto:** ✅ approve (degradado: falta segunda firma de agente distinto; no bloquea porque el entregable es un reporte, sin efecto sobre el build ni sobre el waiver).

## Notas

- **Hallazgo colateral (fuera del blast radius, NO corregido):** `.opencode/AGENTS.md` §"Default Features" afirma
  que `arrow` está en los defaults (`cli + arrow + fjall + …`). `Cargo.toml:156` dice lo contrario
  (`default = ["cli", "fjall", "memmap2", "fs2", "roaring"]`). El waiver de `deny.toml` se apoya en el
  `Cargo.toml` real, así que el waiver es correcto y **el AGENTS.md está desactualizado**. `.opencode/` es
  repo separado y esta tarea no puede editarlo → reportar a `vanta-docs`/owner.
- **Consecuencia operativa del punto anterior:** si alguna vez alguien mete `arrow` en `default`, el build
  publicado arrastra chrono 0.4.45 y la línea "unreachable in the default build" del waiver deja de ser
  cierta. Ese es el invariante a vigilar en la próxima revisión (2027-03-22), no solo la versión de arrow.
- **Por qué chrono quedó sin maintainer:** [chronotope/chrono#1768](https://github.com/chronotope/chrono/issues/1768)
  (referencia del propio advisory). RUSTSEC lo clasifica *Unmaintained* / INFO: no hay CVE ni parche
  porque el problema no es un bug concreto sino la ausencia de maintainer.
- **Por qué arrow lo arrastra:** `arrow-arith` — el módulo de aritmética temporal de arrow-rs — usa
  `chrono::Duration`/`DateTime` en sus kernels de intervalos. chrono entra con la feature `clock`
  habilitada (`arrow-arith` la enciende por default), que es la que trae `now()`/TZ local.
- **Fecha de próxima revisión:** 2027-03-22 (mandatoria, en los tres espejos) **o** en cuanto
  `arrow-arith` publique una versión sin `chrono` en sus dependencies — lo que ocurra primero.
  Chequeo barato del monitor: `cargo tree -i chrono --features arrow` + la API de crates.io de
  `arrow-arith` (última versión).
- **No se ejecutó** `cargo update`, `cargo add`, ni se editó `Cargo.toml`/`Cargo.lock`/`deny.toml` (contrato de monitor).
