---
title: "FIND-736 — the API surface counts impls of private types as public contract"
kind: task
status: completed
description: "extractRust emits `rust impl T` without asking whether T is public, so 46 of 176 impls were contract that does not exist; the extractor now requires a pub declaration or a pub use re-export"
---

# FIND-736 — the API surface counts impl of private types

> **Plan:** `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md`
> **Carril:** C (sin `cargo`) · **Appetite:** 1-2h
> **Dependencias:** `FIND-733` ✅ (`8bc0e57bb`, midió el hallazgo) · `FIND-731` (adyacente, **no** se resuelve acá)

## Contexto

`extractRust()` emite `rust\timpl\t<Tipo>` para **cada** bloque `impl` de nivel
superior sin consultar la visibilidad del tipo. Un `struct` privado con `impl`
produce un item de superficie que el crate no publica: al cerrar `FIND-733` el
gate reportó `+ rust impl llm::EmbedCache` y `+ rust impl llm::WarmLocal` como
superficie nueva cuando ambos son `struct` sin `pub` (`src/llm.rs:240,267`), y
exigió un ADR que no correspondía.

Medido sobre el crate completo antes del arreglo: **46 de 176** items `impl`
(26 %) eran de tipos no públicos.

La fuente de verdad no es este script: es el golden del compilador
(`tests/api/public-api.txt`, generado por `cargo public-api`). El extractor es
un índice derivado y barato; el golden es el contrato. Este cambio **no toca el
golden**.

## Impacto mapeado (Regla 0)

**Leídos completos antes de editar**

| Archivo | Qué se leyó | Por qué |
|---|---|---|
| `scripts/docs/check-api-docs.mjs:1-1380` | los tres extractores, `evaluate()`, `--self-test`, `reportSurface`/`reportChanged` | es el archivo que se modifica; el filtro vive dentro de `extractRust` |
| `scripts/docs/lib.mjs:1-627` | `abs`/`ROOT`, `listDocs`, frontmatter, `deriveKind` | el extractor importa `abs`/`ROOT`; el task file lo valida `check-docs` |
| `.opencode/rules/api-contract.md` | R-1 (todo claim apunta a un símbolo real), R-8 | la regla que el gate sirve: superficie publicada, no implementación |
| `docs/dev/tasks/FIND-733.md` | el hallazgo, sus tres verificaciones y el delta del golden | origen y líneas exactas del defecto |
| `docs/api/index.md` | encabezado `GENERATED` + tabla | confirmar que el índice de `docs/api/` sale de frontmatter, **no** de la superficie (`gen-index.mjs`), así que no lista `impl` |
| `docs/dev/workflow/gate-api-docs.md:90` | cómo se describe el extractor rust | detectar prosa que quede desactualizada (ver §NOTICED) |

**Referencias hacia dentro (¿quién consume la superficie?)**

- `buildSurface` / `surfaceSize`: solo `check-api-docs.mjs` (CLI `--api-surface`,
  `--changed`). Nadie más importa el módulo: otros scripts (`gen-index`,
  `gen-surface-manifest`, `verify-surface-manifest`) leen frontmatter o
  `Cargo.toml`, no la superficie.
- `.github/workflows/docs-gates.yml:391-420`: corre `--self-test`,
  `--api-surface` y `--changed <base>..HEAD`. El ADR Gate es el consumidor real.
- `docs/api/*.md`: ninguna página menciona `impl EmbedCache|WarmLocal`
  (`rg` → 0 hits antes y después).

**Referencias hacia fuera**

- `extractRust` → `Ref.reader` (`read`, `list`, `prefetch`) y `stripRust`. Sin
  cambios de interfaz.
- El nuevo filtro usa `lastSegment()` (ya existente) y el propio `out`. No se
  añaden imports ni dependencias.

**Veredicto de impacto:** 1 archivo de código (`scripts/docs/check-api-docs.mjs`,
solo `extractRust` + su self-test + el comentario de cabecera) + 1 task file +
los índices generados. Cero cambios en `src/**`, `tests/**`, el golden, CI o
hooks. El conteo baja 54 entradas y **no sube ninguna**: el filtro es puramente
sustractivo.

## Steps

1. ✅ Medir el baseline: `--api-surface` → rust 1718 (impl 176, method 490),
   total 2716. Reclasificar los 176 `impl` contra el conjunto de tipos `pub`
   → **46 no públicos**, exactamente la cifra de `FIND-733`.
2. ✅ Verificar que los 46 son de verdad privados: grep de declaración por cada
   tipo → 41 son tipos locales sin `pub`/`pub(crate)`; 5 no son tipos locales
   (3 artefactos de trait-impl sobre `[u8]`/`Vec`, 2 tipos de `std`).
3. ✅ **RED:** caso en `--self-test` — `struct Foo` privado con `impl Foo` no
   entra, `pub struct Bar` con `impl Bar` sí, `pub(crate) struct Qux` no.
   Falla con `impl of a private struct must not be surface` (18/19).
4. ✅ **GREEN:** construir el conjunto de tipos públicos (declaraciones
   `struct|enum|trait|union|type` + `reexport`) al final del walk y filtrar
   `impl`/`method` contra él. 19/19 assertions.
5. ✅ Delta: rust 1718 → 1664 (−54 = 46 `impl` + 8 `method`), total 2716 → 2662,
   **0 added**. Los 8 métodos son de `pub(crate)` (`AlignedBlob` ×4,
   `NodeBytes::as_slice`, `ProcessCryptoGuard::acquire`, `WalVerifyStatus::{as_str,is_ok}`).
6. ✅ Cross-check contra el golden: ninguno de los nombres descartados aparece
   como item del crate en `tests/api/public-api.txt` (solo `Mutex`/`RwLock`/`Vec`
   aparecen, como tipos de `std` en firmas ajenas). Golden **sin tocar**.

## Contrato

```powershell
node scripts/docs/check-api-docs.mjs --self-test        # 19/19, incluye private/pub
node scripts/docs/check-api-docs.mjs --api-surface      # rust 1664, impl 130
rg -n 'impl .*::(EmbedCache|WarmLocal)' docs/api/ -g '*.md'   # 0 hits

# El hallazgo, medido de punta a punta: el rango que reportaba +3 ahora da +1
node scripts/docs/check-api-docs.mjs --changed 06485fe8e..HEAD

# No-dependencia de FIND-731: el filtro es aritmética sobre el árbol leído.
# No consulta VANTADB_PUBLIC_API_UPDATE ni el golden.
```

## Verificación

| Gate | Comando | Resultado |
|---|---|---|
| Self-test (private pub pair) | `node scripts/docs/check-api-docs.mjs --self-test` | ✅ 19/19 (nuevo caso incluido) |
| Superficie nueva | `node scripts/docs/check-api-docs.mjs --api-surface` | ✅ rust **1664** (impl 130, method 482), total **2662** |
| Hallazgo cerrado | `--changed 06485fe8e..HEAD` | ✅ `+1` (`executor::parse_iql`), antes `+3` |
| Sin remociones fantasma | `--changed HEAD..WORKTREE` | ✅ `surface unchanged` (el filtro se aplica igual a base y head) |
| Índice | `rg 'impl .*::(EmbedCache\|WarmLocal)' docs/api/` | ✅ 0 hits |
| Golden intacto | `git status tests/api/public-api.txt` | ✅ sin cambios |
| Docs | `check-docs` → `GATING: all clear` · `check-links` → dentro del budget | ✅ |
| Índices generados | `gen-index.mjs --write` + `--check` | ✅ |
| Review P2-01 | `vanta-review` (contexto fresco ≠ autor) | ver §Review |

## Delta (46 → 0 `impl` de tipos privados)

`impl` 176 → **130**; `method` 490 → **482**; rust 1718 → **1664**; total
2716 → **2662**. Los 46 descartados, por tipo:

```
cli_handlers::encryption::ChildGuard, EnvVarGuard
config::EnvGuard, config::RedactedTokenRoleMap
crypto::Aes256KeyType, ProcessCryptoGuard
crypto::registry::FailpointCleanup, Reader
entity::linking::DisjointSet
executor::AllocationGuard, CaptureWriter, ProviderEnvGuard
llm::EmbedCache, EnvGuard, KeyGuard, UnconfiguredLocalProvider, WarmLocal
migration::ArtifactSpec
operator_registry::MockScan, PassthroughCompiler, PassthroughCost
physical_plan::MockScan
planner::MockRrfArm, PhysicalRrfFusion
server::bootstrap::CaptureWriter, FakeService, HungService, OrderProbe,
  PanicService, TracingMarkerService
server::conversation::FakePorts · server::handlers::ThreadDTO
server::list_records::FakePorts
storage::vfile::AlignedBlob, DecryptCache, FileMap, NodeBytes
sync_ext::Mutex, RwLock          <- tipos de std en trait impls del crate
tui::TerminalGuard
vector::governor::AccessEntry
wal::WalVerifyStatus
wiki::ingestors::FooIngestor
crypto::Vec, crypto::[u8;, crypto::[u8]   <- artefactos de `impl AsRef<[u8]> for Vec<u8>`
```

Familias: guards de recurso/entorno (9), dobles de test y `Mock*`/`Fake*`/
`Passthrough*` (12), servicios E2E de `server::bootstrap` (6), helpers privados
del motor (13), artefactos no-tipo (5).

## Falsos negativos: cómo se acota el riesgo

Un extractor demasiado estricto **oculta** un `impl` público, que es peor que
contar uno privado. Decisiones que lo acotan:

- El conjunto de tipos públicos incluye los **re-exportados** (`pub use`), no
  solo los declarados en el módulo del `impl`. Cubierto por el self-test con un
  `impl crate::external::Widget` sobre un tipo que solo lib.rs nombra en un
  `pub use` (quitando la rama `reexport` del filtro, ese caso falla).
- La comparación es por **último segmento**, así `impl crate::a::Foo` escrito en
  otro archivo se resuelve sin un resolver de módulos. Coste declarado: un `Foo`
  privado puede heredar el veredicto de un `Foo` público — falso **positivo**,
  el lado correcto del error. Instancia viva hoy: `rust impl Shell` sobrevive
  por colisión con el `pub enum cli::Shell` (`src/cli.rs:615`), aunque el `impl`
  es `From<Shell> for clap_complete::Shell` (`:638`). Inofensivo.
- `pub(crate)`/`pub(super)` no entran al conjunto: el regex de declaración exige
  `pub` + espacio, y el filtro descarta lo que no está en el conjunto. Los 4
  tipos `pub(crate)` que perdieron entradas (`AlignedBlob`, `NodeBytes`,
  `WalVerifyStatus`, `ProcessCryptoGuard`) viven en `pub mod` **sí recorridos**;
  los descarta el filtro, no un hueco del walk.

**Clases que este cambio sí puede ocultar** (declaradas, no barridas bajo la
alfombra):

1. **`impl` escrito a mano sobre un tipo `pub` generado por macro.** El regex de
   declaración no ve un `pub struct` emitido dentro de `macro_rules!`, así que
   su `impl` (que sí es texto plano) cae ahora. Antes se contaba. **Medido: 0
   instancias** — los 46 descartados resuelven todos a una declaración real no
   `pub` (41) o a un artefacto no-tipo (`crypto::{Vec,[u8;,[u8]}`, `sync_ext::{Mutex,RwLock}`).
   El golden del compilador, que sí ve macros, tampoco lista ninguno.
2. **Re-export glob (`pub use m::*`).** `flattenUse` devuelve `*`, no los
   nombres, así que un tipo declarado fuera del walk y publicado solo por glob
   no entra al conjunto. **Medido: 0 instancias** — hay 33 globs en 8 archivos;
   los 10 que viven en módulos que el walk no recorre
   (`src/index/distance/mod.rs:12-13`, `src/index/graph/mod.rs:10-12`,
   `src/metrics/core/mod.rs:9-10`, todos bajo `pub(crate) mod`) nunca aportan
   tipos a la superficie, y los de `src/index/mod.rs:20-21`, `src/metrics/mod.rs:9`,
   `src/parser/mod.rs:8-9` re-exportan *desde* esos mismos `pub(crate) mod`.
   Ningún nombre descartado sale de esos árboles y el golden no lista ninguno
   por esa vía. Arreglarlo exige resolver el módulo glob; no se hace acá.

**Medición del residuo:** de los 130 `impl` que sobreviven, **38** no aparecen en
el golden. No son privados: son tipos `pub` detrás de features no-default
(`crypto`/`encryption`, `server`, `python_sdk`, `llm`/`remote-inference`,
`testing`) que el golden —generado con features default— no lista. Era el
comportamiento previo y el filtro los conserva: la lista de tipos públicos se
deriva del árbol leído, no del golden.

## Definition of Done

- Nivel **Task** (tooling de docs, sin `cargo`).
- [x] RED → GREEN con el caso privado/público en `--self-test` (19/19); el caso
  falla si se quita el filtro (18/19) y si se quita la rama `reexport` (18/19).
- [x] Delta medido y documentado (46 → 0), `0 added`.
- [x] Cross-check contra el golden del compilador; golden sin tocar.
- [x] `src/**`, `tests/**`, `.github/**`, `.githooks/**`, Backlog, plans y avance
  sin tocar **por esta tarea**: `git status` no lo puede probar solo en un
  worktree compartido, donde `tests/api/public_api.rs` (fix de `FIND-731`),
  `vanta-memory/tests/conversation_hook.rs`, `vantadb-mcp/src/threads.rs`
  (MCP-44/MEM-71) están sucios por otros carriles. El golden
  `tests/api/public-api.txt` no aparece en `git status`.
- [x] Review P2-01 por agente distinto.
- **Release: no aplicable** — `scripts/docs/**` no está en el grafo de publish.
  Sin entrada de `CHANGELOG.md`: la Regla 7 lo prohíbe fuera de release-plz.

## NOTICED BUT NOT TOUCHING

- **`docs/dev/workflow/gate-api-docs.md:90`** describe el extractor rust como
  "métodos de `impl` inherentes" sin la regla de visibilidad, y cita **1300**
  cuando el conteo ya era 1718 (hoy 1664). Fuera del whitelist de escritura de
  esta tarea → fila `FIND-*` candidata.
- **Árbol compartido:** `gen-index.mjs --write` (artefacto derivado, nunca a
  mano) indexó 775 documentos: además de `FIND-736.md`, entraron las filas de
  `MCP-44.md` y `MEM-71.md`, task files de otros carriles en vuelo en el mismo
  worktree (sin trackear). Van en el commit del índice porque el generador es la
  única fuente y editar el resultado a mano está prohibido. Consecuencia
  declarada: **en un checkout de este commit solo**, esas dos filas son enlaces
  a ficheros ausentes (`check-links` los cuenta, budget 0). Se resuelve solo si
  `MCP-44` y `MEM-71` aterrizan en el mismo push o antes; si no, el arreglo es
  `node scripts/docs/gen-index.mjs --write` después de esos commits. Mismo
  comportamiento ya documentado en `FIND-733`.
- **`FIND-731`** (golden falso-verde con `VANTADB_PUBLIC_API_UPDATE` vacío)
  sigue abierto y es ortogonal: este cambio no lee esa variable ni el golden.
- **`--api-surface` no distingue `impl` inherente de trait impl**: registra
  `${tipo}::${fn}` para cualquier `impl`, incluido `impl Trait for Tipo`. La
  visibilidad ya filtra los privados; la clase de `impl` sigue sin modelarse.
- **Tipo `pub` re-exportado desde un módulo privado:** el walk solo encola
  `pub mod`, así que `mod internal;` + `pub use internal::Thing;` publica el
  nombre (`reexport`) pero nunca lee `src/internal.rs`, y sus `impl`/métodos no
  aparecen. Es un falso **negativo** pre-existente (no lo introduce este
  cambio: antes tampoco se leía el archivo) y arreglarlo implica recorrer
  módulos privados y decidir qué hacer con sus items — otro contrato.

## Review

**P2-01 (ronda 1, `vanta-review`, contexto fresco ≠ autor): `changes-required`.**
El código se aprobó como correcto y todas las cifras se reprodujeron
independientemente; el envío fue por dos defectos del **registro** (prosa y
cobertura del test), no del comportamiento.

Lo que el reviewer verificó por su cuenta (no copiado del reporte):

- Filtro quitado → **18/19** fallando exactamente el caso del tipo privado;
  script de HEAD → 18/18. El caso nuevo es RED-capaz, no una paráfrasis.
- Viejo vs nuevo sobre el mismo árbol: rust 1718 → 1664, remociones 54 = 46 + 8,
  adiciones 0; python/ts intactos.
- `--changed 06485fe8e..HEAD`: viejo **+3**, nuevo **+1**.
- Caza de falsos negativos por dos vías independientes (golden de 11 168 líneas
  + scan de declaraciones sobre todo `src/`): **0** falsos negativos en `HEAD`.
- Consumidores: solo `docs-gates.yml`; ningún módulo importa el script.

| # | Sev | Hallazgo | Disposición |
|---|---|---|---|
| F1 | Medium | La frase "los items generados por macro tampoco se contaban antes: no hay regresión" es falsa: un `impl` escrito a mano sobre un `pub struct` generado por macro **sí** se contaba y ahora cae | ✅ §Falsos negativos ahora **nombra la clase** y la medición que la encontró ausente (46/46 resuelven a declaración real no `pub` o artefacto no-tipo; el golden tampoco lista ninguno) |
| F3 | Medium | El self-test no probaba "el conjunto se arma después del walk": `inner` (declaración) se visitaba antes que `other` (impl), así que un filtro por archivo pasaba igual | ✅ `pub mod other;` ahora precede a `pub mod inner;` — el impl se encuentra antes que su declaración, y el assert del método cruzado falla con un filtro por archivo |
| F4 | Low | La rama `reexport` era no-op y no tenía test (borrarla dejaba 19/19) | ✅ Se conserva porque el contrato la exige, y ahora es **load-bearing**: fixture `impl crate::external::Widget` + `pub use external::Widget`. Mutación comprobada: sin la rama → **18/19** |
| F2 | Low | Los re-export glob (`pub use m::*`) no pueblan el conjunto (`flattenUse` → `*`) | ✅ Declarado como clase de falso negativo nº 2, con medición de instancias (0: los ~20 glob cuelgan de `pub mod` recorridos). Arreglarlo exige resolver el módulo; fuera de alcance |
| F6 | Low | La explicación de los 4 `pub(crate)` decía "sus módulos tampoco se recorren" — falso: están en `pub mod` recorridos | ✅ Corregido: los descarta **el filtro**, no un hueco del walk |
| F5 | Low | El índice regenerado publica filas de `MCP-44`/`MEM-71` (sin trackear); en un checkout del commit solo, `check-links` vería 2 enlaces colgantes | ⚠️ Aceptado con consecuencia escrita (§NOTICED): el generador es la única fuente y editar a mano está prohibido. Se cierra regenerando tras esos commits (instrucción al orquestador en el RESULTADO) |
| F7 | Nit | Sin entrada `[Unreleased] → Fixed` en `CHANGELOG.md` | ❌ Rechazado: la Regla 7 prohíbe tocarlo a mano; lo mantiene release-plz (única excepción: mantenimiento estructural aprobado por el owner) |
| F8 | Nit | `rust impl Shell` sobrevive por colisión de nombre con `pub enum cli::Shell` (`src/cli.rs:615`) sobre un `impl From<Shell> for clap_complete::Shell` (`:638`) | ⚠️ Aceptado: es la instancia viva de la clase de falso positivo documentada; se cita en §Falsos negativos, sin acción |

Alternativas que el reviewer evaluó y descartó (registradas para que no se
re-propongan): resolver por path de módulo (necesita un mini-resolver Rust para
~40 líneas de ganancia), usar el golden como oráculo de visibilidad (perdería los
38 `pub` feature-gated y `FIND-731` prueba que puede quedar stale), y borrar por
regla los trait impls sobre tipos foráneos (mejora opcional: hoy `Vec`/`[u8]`/
`Mutex`/`Shell` entran por accidente, pero esas entradas nunca aportaron poder
de detección).

**P2-01 (ronda 2, mismo reviewer, verificación de disposiciones): `approve`.**
Los 6 hallazgos abiertos cerrados con evidencia de mutación que falla en el
assert exacto: per-file filter → 18/19 (`an impl written in another module
resolves against the declaration`); sin la rama `reexport` → 18/19 (`a pub use
re-export makes the type public even when its declaration is not walked`).
F7 retirado (la regla de la Regla 7 existe verbatim: `.opencode/AGENTS.md:641,671`).
El bloque del filtro quedó byte-idéntico al de la ronda 1 (sin drift de
comportamiento). Nits de la ronda 2, todos aplicados: **N1** el recuento de
"declaración no `pub`" era 41, no 42 (46 − 5 artefactos); **N2** la enumeración
de globs se corrigió a la forma real (33 globs en 8 archivos, 10 en módulos
`pub(crate)` no recorridos; la conclusión de 0 instancias se mantiene);
**N3** faltaba el assert simétrico del **método** de un tipo `pub(crate)`
(`inner::Qux::also_dropped`), ahora presente.

**Veredicto final: approve.** Pendiente solo de orquestación: `docs/index.md`
lleva las filas de `MCP-44`/`MEM-71` (ver §NOTICED).
