---
title: "FIND-219 - la matriz de features se publica, y el build self-hosted que nadie compilaba gana su gate"
kind: task
status: completed
description: "27 flags binarios producen 2^27 combinaciones; el CI compilaba 6 y ninguna era el build self-hosted que DEPLOYMENT_GUIDE:65 le dice al operador. Se publica docs/feature-matrix.md con la tabla de combinaciones y una columna 'probado en CI' que dice la verdad, y se agrega un job `check` (~2 min) para esa combinación."
tags: [vantadb, cargo, features, ci, github-actions, docs]
---

# FIND-219 - 27 flags, 6 combinaciones compiladas, cero gates para la que se documenta

> **Carril:** documentación + CI, **sin compilar Rust** (disco ~19 GB). Escritura:
> `docs/feature-matrix.md` (nuevo) · `.github/workflows/ci-rust.yml` · este task file ·
> `docs/dev/Backlog.md` (sólo la fila `FIND-219`) · `docs/dev/avance/activo/ci-cd.md`.
> **Prohibido y respetado:** `src/**`, `Cargo.toml`/`Cargo.lock`, `vantadb-mcp/**`,
> `vanta-memory/**`, `dev-tools/**`, las otras lanes de certificación, `.opencode/**`,
> `docs/dev/plans/**`. **`gen-index` no se corrió** (ver §Lo que queda abierto).

## Contexto (medido contra el árbol de hoy, no heredado de la fila)

La fila decía "16+ features binarias". Son **27** en el crate core (`Cargo.toml:155-199`
= `default` + 26 opt-in), más 8 en `vanta-memory`, 8 en `vantadb-server`, 3 en
`vantadb-wasm`, 2 en `vantadb-mcp`: **48 nombres declarados**. 2²⁷ combinaciones
teóricas — no es un número a cubrir, es la razón para publicar la lista corta.

Lo que el CI compila hoy, leído job por job de `ci-rust.yml` (no supuesto):

| Gate | Features | Runtime |
| --- | --- | --- |
| `Clippy Lints` | `--all-features` = **la unión**, una sola combinación | no |
| `Tests (Linux/Windows/macOS)` | `cli,arrow,tls,opentelemetry` + default | sí, 3 OS |
| `Tests (embeddings features)` | la anterior + `remote-inference` **o** `embed-local` (`--lib`) | sí, 2 legs |
| `Miri (UB Detection)` | `--no-default-features --features "cli,fjall,memmap2,fs2,encryption"` | sí |
| `Release Combo` | `server,jemalloc` (+ default), perfil release, `-D warnings` | no |
| `Code Coverage` / `ASan` | = la lane de tests | sí (coverage sí, ASan best-effort) |
| `WASM (wasm-pack browser tests)` | core `--no-default-features --features wasm`, target wasm32 | sí, path-gated y `continue-on-error` |

**Dos huecos reales, no estimados:**

1. `DEPLOYMENT_GUIDE.md:65` le dice al operador
   `--features "cli,server,tls,opentelemetry,advanced-tokenizer,remote-inference"` — **ningún
   job compila ese conjunto**. No es subconjunto de la lane de tests: ésa lleva `arrow` en
   lugar de `advanced-tokenizer`, así que la combinación con el camino columnar **apagado**
   y `server` **encendido** nunca se compila; y `clippy --all-features` no puede verla
   porque las features son aditivas.
2. `release-binaries.yml:120-127` compila los binarios publicados con
   `server,embed-local,<allocator>`, pero `Release Combo` (`ci-rust.yml:972,975`) chequea
   `server,jemalloc` **sin `embed-local`**. El gate que existe justamente para no repetir
   `FIND-229`/`FIND-231` ya no está en paridad con la lane que publica.

**Y dos afirmaciones falsas en el registro vigente**, medidas:

- `FEATURES.md:32` declara `default` = `cli, arrow, fjall, roaring, advanced-tokenizer,
  memmap2, fs2, sysinfo, rayon`. El real (`Cargo.toml:156`) es `cli, fjall, memmap2, fs2,
  roaring` — **cinco**, no nueve. El mismo doc cita `ci-rust-10.yml`, que no existe.
- `FEATURES.md:51` dice que `vantadb-python` consume el core con `default-features=false`.
  `vantadb-python/Cargo.toml:25` no lo hace: el wheel se construye con los defaults
  encendidos + `fjall,memmap2,rayon` → efecto neto `default + rayon`.

No se tocan (fila nueva, no rediseño de esta): quedan declaradas en
`docs/feature-matrix.md` §1 con el `file:line` que las contradice.

## Impacto mapeado (Regla 0)

Leídos completos antes de editar: `Cargo.toml:1-99` y `:100-230` (features + deps),
`[features]` de `vanta-memory`, `vantadb-mcp`, `vantadb-wasm`, `vantadb-server`,
`vantadb-python/Cargo.toml` (completo), `.github/workflows/ci-rust.yml` (jobs 1-200,
222-331, 431-720, 726-1021, 1060-1240), `release-binaries.yml:99-135`,
`docs/dev/architecture/FEATURES.md` (completo), `docs/user/operations/EXPERIMENTAL_FEATURES.md:1-60`,
`src/bin/vanta-cli.rs` (gates de allocator), `scripts/validate-frontier.ps1:1-70`,
`scripts/docs/check-docs.mjs`/`gen-index.mjs` (contrato de `kind` y de índices).

- **Referencias hacia dentro:** `docs/feature-matrix.md` no existía (0 inbound). El job nuevo
  es aditivo: ningún `needs:` lo referencia, ningún nombre de context existente cambia.
- **Referencias hacia fuera:** el doc enlaza sólo a `docs/**.md` existentes
  (`dev/architecture/FEATURES.md`, `user/operations/EXPERIMENTAL_FEATURES.md`,
  `dev/operations/CI_POLICY.md`, `user/operations/DEPLOYMENT_GUIDE.md`) — verificado con
  `check-links` (0 broken).
- **Veredicto:** el único riesgo de acoplamiento es el índice generado (`gen-index --check`,
  required) — medido y declarado abajo, no ocultado.

## Steps

| # | Step | Estado |
| --- | --- | --- |
| 1 | Inventario de features por crate contra los `Cargo.toml` (27/8/8/3/2) y censo de lo que compila cada job | ✅ |
| 2 | Elegir el conjunto soportado (6 combinaciones + las superficies python/wasm) con su razón; descartar `minimal-cli` por no ser promesa de nadie | ✅ |
| 3 | `docs/feature-matrix.md` con la tabla, la columna "covered" veraz, las combinaciones no soportadas y el ledger de coste | ✅ |
| 4 | Job `Feature Combo (server-full)` en `ci-rust.yml`: `check` acotado, `needs: [fmt, clippy]`, sin `continue-on-error` | ✅ |
| 5 | `actionlint` con y sin shellcheck, exit 0 y sin cuelgue (medido) | ✅ |
| 6 | Cierre: fila del Backlog, avance en `ci-cd.md`, commit por el helper | ✅ |

## Verificación (comandos exactos y salida)

| Comando | Resultado |
| --- | --- |
| `actionlint.exe .github/workflows/ci-rust.yml` | exit **0**, **1.34 s** (baseline HEAD 1.25 s) |
| `actionlint.exe -shellcheck= .github/workflows/ci-rust.yml` | exit **0**, **0.14 s** (baseline 0.09 s) |
| `node scripts/docs/check-links.mjs` | exit **0**, 0 broken (3.643 links, 1 wikilink presupuestado, ajeno) |
| `node scripts/docs/check-docs.mjs` | exit **0**; `docs/feature-matrix.md` aparece como *orphan* (aviso, no gate) |
| `node scripts/docs/gen-index.mjs --check` | **exit 1 — declarado**, ver abajo |

`actionlint` no se cuelga: el job nuevo no añade sentencias a un `run:` existente; son dos
`run:` de una sentencia cada uno (el `checkout`/`setup` son `uses:`).

## Lo que queda abierto (declarado, no escondido)

- **`gen-index`: no se corrió acá, y ya está resuelto por otro carril.** Al escribir los
  documentos el índice quedó stale, y `--write` habría indexado también ficheros ajenos
  del mismo worktree (clase `FIND-792`), así que se declaró. Otro carril regeneró el
  índice en `3fd8f253c` y **capturó estos dos ficheros**: `docs/index.md:23,648` y
  `llms.txt:101` ya los listan, y `node scripts/docs/gen-index.mjs --check` devuelve
  **exit 0** medido después. `Generated indexes are current` (required #13) queda verde.
  El frontmatter de este task file y el de la matriz se congelan: la fila del índice
  incluye `title` + `description`, y cambiarlas acá volvería a ponerlo rojo sin poder
  regenerarlo en este contrato.
- **`embed-local` fuera del `Release Combo`: declarado, no arreglado.** Es un cambio de una
  cadena (`ci-rust.yml:972,975`), pero mueve el perfil release bajo `-D warnings`, que es
  exactamente donde `FIND-231` encontró el bug de 0.8.0. Sin poder compilar release acá, un
  rojo sería una lane bloqueada sin diagnóstico. La matriz lo publica como 🟡 con el
  arreglo exacto: lo decide quien pueda compilarlo.
- **`compile_error!` para combinaciones no soportadas: no se escribe.** No hay ninguno en
  el árbol (0 hits en `src/`, `vantadb-wasm/src/`, `vantadb-server/src/`), y agregarlo
  necesita `src/bin/vanta-cli.rs` (prohibido en esta fila). La matriz documenta cómo
  resuelve hoy cada combinación (el par de allocators se resuelve por `cfg`, no por error).
- **`server` sin `cli`:** `FEATURES.md:48` afirma que `server` requiere `cli`; el
  `Cargo.toml` no lo declara y ningún gate lo mide. Queda como ⚠️ **no verificado**, no
  como claim.
- **El job nuevo no está en el ruleset** (decisión del owner, igual que `tests-guard` antes
  de estarlo): un rojo ahí cuesta una mirada, no un merge bloqueado.

## Review P2-01

Tier **fast** (paths: `docs/**` + `.github/workflows/**` → "cualquier otro path"). El
veredicto adversarial va en el bloque RESULTADO de la sesión que ejecutó la tarea.
