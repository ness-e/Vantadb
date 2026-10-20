---
title: "Deep audit W9 - Cross-cutting: security, concurrency, memory, API contract"
kind: review
status: active
description: "Auditoria transversal de VantaDB: unsafe/UB, concurrencia y orden de locks, presupuesto de memoria, contrato de API y semver, paridad de scores entre SDKs y gates que fallan de forma incorrecta."
tags: [audit, reviews, cross-cutting]
---

# Deep audit W9 - Cross-cutting

> **Wave:** W9 · **Fecha:** 2026-10-20 · **Modo:** read-only
> **Especificacion:** `docs/dev/plans/2026-10-20-revision-profunda-integral-vantadb.md` §4 (W9)
> **Ejecucion:** subagentes `vanta-audit` + `vanta-chaos` no disponibles (sin tool `task`).
> **Input adicional:** `docs/dev/reviews/2026-10-20-release-v0.9.0-postmortem.md` (publicado el
> 2026-10-20, mismo dia de esta revision) - 7 fallos encadenados de publicacion, ninguno detectado
> por un gate.

## Scope

**Cubierto:** `src/**` (UB/unsafe, concurrencia, memory budget), `src/error.rs` + `src/sdk/**`
(contrato de API), `src/rbac.rs`, `src/server/{jwt,router,middleware}.rs`, `src/api/scores.rs`,
`benches/scores_semantics.rs`, `.github/workflows/ci-rust.yml` (ADR gate),
`.github/workflows/{certification,nightly}.yml` (fuzz/chaos).

**Excluido con motivo:** `fuzz/` (fuera del workspace, ver `Cargo.toml:784-786`); el detalle de
seguridad de los bindings (W2).

## Metodologia

- Analisis estatico de `unsafe` y su documentacion (ver W1).
- Lectura del postmortem de 0.9.0 como fuente de fallos de gate verificados en produccion.
- Verificacion de la propagacion de los arreglos de ese postmortem en los workflows actuales.
- Sin ejecucion (cargo ausente del PATH).

## Hallazgos

| ID | Severidad | Hallazgo | Archivo:línea | Evidencia | Fix propuesto | Esfuerzo |
|---|---|---|---|---|---|---|
| `FIND-405` | Medium | **El ADR Gate falla por cambios que no son API publica y bloquea PRs validos** | `.github/workflows/ci-rust.yml:167-176` | Postmortem de 0.9.0, F7: "El filtro de paths del gate es `vantadb-ts/` (todo el arbol), no la superficie publica real: un **script de smoke test** no es API publica, pero el gate lo trata como tal. Falso positivo." El propio postmortem marca el arreglo de fondo como "**PENDIENTE**". El gate esta descrito en `ci-rust.yml:168-173` como "compares the PR changeset against the base SHA and fails when public API files changed unless an ADR (docs/dev/architecture/adr/ADR-NNN-*.md) was added OR the PR is explicitly marked [no-adr]". El filtro de paths usa el arbol del paquete, no `docs/api/SURFACE_MANIFEST.md` ni los ficheros de superficie publica que el propio repo genera (`scripts/docs/gen-surface-manifest.mjs`). Workaround actual: marcar `[no-adr]` en el PR - es decir, **desactivar el gate a mano** en vez de arreglarlo. | Cambiar el filtro para que use la superficie publica real (los `.d.ts`/`.pyi`/ficheros de API que `gen-surface-manifest.mjs` ya inventaria) en lugar de `vantadb-ts/`. Cerrar el "PENDIENTE" del postmortem. | 1d |
| `FIND-406` | Medium | **La politica de publicacion documentada en AGENTS.md es falsa hoy** | `AGENTS.md:646-648` | `AGENTS.md:646-648` afirma: "**Publish 100% tokenless vía Trusted Publishing (OIDC)** - verificado 2026-10-19: ningun workflow referencia `secrets.*` para publicar · `gh secret list` vacio a nivel repo, correcto). No crear estos tokens." Verificado el 2026-10-20 con `gh secret list`: existen **5 secrets** (`CARGO_REGISTRY_TOKEN` 2026-10-20T04:46Z, `NPM_TOKEN` 2026-10-20T07:04Z, `PYPI_TOKEN` 2026-10-20T04:46Z, `RELEASE_PLZ_TOKEN` 2026-10-20T04:46Z, `TEST_PYPI_TOKEN` 2026-10-20T05:00Z) y **12 lineas** de workflow los referencian: `release-wheels.yml:172,214`, `release-adapters.yml:128,176`, `release-providers.yml:195,280`, `release-npm-61.yml:250,340`, `release-npm-node.yml:258`, `release.yml:55` (`RELEASE_PLZ_TOKEN`). La afirmacion es **parcialmente** cierta (crates.io para `vantadb` si usa OIDC) pero **falsa como generalidad**, y el propio `release.yml:56-71` documenta que `vanta-memory` necesita token porque Trusted Publishing exige que el crate ya exista. Consecuencia: un agente que siga AGENTS.md creera que publicar es tokenless y **no creara** los tokens que los 5 workflows necesitan - que es exactamente el fallo F1/F2 del postmortem ("please provide a non-empty token", "403 Forbidden: authentication failed"). | Reescribir `AGENTS.md:646-648` con la realidad: crates.io (`vantadb`, `vantadb-wasm`, `vanta-proxy`) via OIDC; PyPI (wheels/adapters/providers) y npm via token con nombre de registry; `vanta-memory` primer publish via token. Enlazar `docs/dev/workflow/PUBLISH.md`. | 1h |
| `FIND-407` | Critical | **`SECURITY.md` publica una garantia de cifrado en reposo que el producto no ofrece por defecto ni sobre el WAL** (refutado **parcialmente** en W10: `vanta-proxy` si habilita `vantadb/encryption` y `vanta-memory` lo expone via la feature `erasure`; el defecto persiste para el crate `vantadb` publicado en crates.io y para la WAL/journal en **todos** los canales) | `SECURITY.md:26`, `Cargo.toml:178`, `src/storage/vfile.rs:126`, `src/storage/vfile.rs:439` | `SECURITY.md:26` lista bajo "Security Practices": "**AES-256-GCM at-rest encryption** behind the `encryption` feature flag (`src/crypto.rs`)" - sin ninguna advertencia. Verificado: (1) `Cargo.toml` declara `default = ["cli", "arrow", "fjall", "roaring", "advanced-tokenizer", "memmap2", "fs2", "sysinfo", "rayon"]` - **`encryption` NO esta en default** (`Cargo.toml:178` la declara como feature opcional: `encryption = ["dep:aes-gcm", "dep:ring"]`). El binario publicado no cifra nada. (2) El cifrado solo se engancha a la capa vstore: `src/storage/vfile.rs:126` (`pub cipher: Option<Cipher>`) y `:439` (`pub fn with_cipher`). Busqueda de `Cipher|EncryptionStream|encrypt` en `src/wal.rs` y `src/wal_sharded.rs` = **0 ocurrencias**: **la WAL no se cifra**. El journal Fjall (`0.jnl`) tampoco. Es decir, la afirmacion es doblemente incompleta: opt-in y parcial. Consecuencia concreta y ya medida: los payloads Write-Ahead-Log quedan **en claro en disco**, que es exactamente el residuo documentado en `tests/certified_delete.rs:254-256` ("payload legible en `data/vanta.shard0.wal`"). El postmortem de 0.9.0 lo marca como P0 abierto: el tarball de crates.io 0.9.0 **ya esta publicado con el claim**, asi que la correccion solo puede entrar en 0.9.1. Un documento de politica de seguridad no puede afirmar una propiedad que el binario distribuido no tiene. | Reescribir `SECURITY.md:26` para declarar el alcance exacto: "opt-in (feature `encryption`, **fuera de default**); cubre los data files de vstore; **no cubre la WAL ni el journal**", y anadir la lista explicita de lo que no se cifra. En paralelo, registrar el cifrado de WAL como item de roadmap (no como garantia actual). | 2h (doc) + backlog (WAL) |
| `W9-03` | Info | Superficie `unsafe` del sistema: 83 bloques en 24 ficheros, 21 de 24 con invariante `SAFETY` | `src/storage/vfile_mmap.rs:1`, `src/index/distance/kernels.rs:1`, `src/storage/engine/txn.rs:612` | Detalle completo en W1 y W2. Ningun binding de usuario tiene `unsafe` (W2-01). El unico bloque sin invariante es `src/storage/engine/txn.rs:612` (`FIND-396`). `src/index/search/tests.rs:856-868` documenta honestamente **por que** 4 bloques de `from_raw_parts` en `search.rs` no son verificables por Miri (requieren mmap real) y cubre el camino equivalente con `vector_store = None`. | Sin accion adicional. | - |
| `W9-04` | Info | Miri y fuzz: cobertura **parcial y condicionada al host**, no ausente | `.github/workflows/certification.yml:812`, `Cargo.toml:784-786` | `Cargo.toml:784-786`: "fuzz/ is intentionally outside the workspace: cargo-fuzz requires nightly + Linux sanitizers". `certification.yml:812` documenta: "Ubuntu-only (Windows da flaky en fuzz); timeout <= 15 min; sin continue-on-error (Regla 2)". Existe ademas el target `[[test]] fuzz_proptest` (`Cargo.toml:663-664`) que **si** corre en el Fast Gate (proptest, sin sanitizers) - es la red de fuzzing en la PR. La capa de sanitizers (Miri/ASan) es Heavy y Linux-only. | Sin accion. Correctamente estratificado. | - |
| `W9-05` | Info | El contrato de score tiene su fuente unica y su benchmark, pero el benchmark no se ejecuta | `src/api/scores.rs:1`, `benches/scores_semantics.rs`, `docs/api/scores.md:1` | `src/api/scores.rs:1` -> "//! Score semantics helpers - canonical scoring contract for VantaDB." con `rrf_contribution` (linea 17) y `rrf_contribution_0based` (linea 27). `benches/scores_semantics.rs` existe (dataset determinista xorshift seed `0x9E37_79B9_7F4A_7C15`, documentado en `BENCHMARKS.md:292`) y `BENCHMARKS.md:272` documenta el comando `cargo bench -p vantadb --bench scores_semantics`. **Pero ese bench no se invoca en ningun workflow** (ver `FIND-395` en W7). Es decir: la invariante de scoring mas importante del producto tiene un benchmark que nadie ejecuta, mientras el doc que la describe (`docs/api/scores.md`) tiene 7 citas rotas (`FIND-387`). | Anadir `scores_semantics` al Fast Gate (es determinista y barato) y sincronizar `docs/api/scores.md` con el codigo. | 4h |
| `W9-06` | Info | `#[non_exhaustive]` + ADR gate + `version_coherence` = 3 capas de proteccion del contrato de API | `src/error.rs:136`, `.github/workflows/ci-rust.yml:167-176`, `Cargo.toml:591-592` | (1) 17 enums publicos marcados `#[non_exhaustive]` (W1-03). (2) ADR gate que exige ADR ante cambios de API publica (con el defecto de `FIND-405`). (3) target `version_coherence` (`Cargo.toml:591-592`) invocado explicitamente por `release-wheels.yml:95` (`cargo test --test version_coherence --no-default-features --features fjall`). (4) `tests/api/public_api.rs` con golden file `tests/api/public-api.txt` y job de nightly rustdoc (`ci-rust.yml:165`). La cadena es buena; lo que falla es la granularidad del gate, no su ausencia. | Sin accion (ver `FIND-405`). | - |
| `W9-07` | Info | **Los metodos de verificacion que mienten estan documentados** - el postmortem es la fuente | `docs/dev/reviews/2026-10-20-release-v0.9.0-postmortem.md:200-210` | Tabla "Fallos del propio instrumental de verificacion": `cargo owner --list` no autentica (dio PASS a un token que crates.io rechazo con 403); `gh api check-runs?per_page=100` trunca en 100 y **oculta 5 checks requeridos** (un commit con 128 check-runs); `Invoke-WebRequest -Form` a PyPI devuelve 405 (enrutado) en vez de 400/403; el dump de `env` de un job **no incluye** las variables OIDC. Regla general que se deduce: "un token solo se da por valido si un publish real lo acepta. Ningun endpoint de lectura sirve como prueba." Es la clase de hallazgo mas importante de toda la revision: **el fallo no es del codigo, es del metodo de verificacion**. | Incorporar la tabla al runbook de verificacion de release (`docs/dev/workflow/PUBLISH.md`) como checklist de "metodos que mienten". | 2h |
| `W9-08` | Low | `rust-toolchain.toml` (canal 1.95.0) y el `stable` de los workflows pueden colisionar | `rust-toolchain.toml:2`, `.github/workflows/release-adapters.yml` | Postmortem F4: el `rust-toolchain.toml` de la raiz (canal 1.95.0, `components = ["rustfmt","clippy","rust-src"]`) hace que rustup intente instalar el clippy de 1.95.0, cuyo proxy `bin/cargo-clippy` ya esta ocupado por el toolchain `stable` instalado por el action. Arreglo aplicado: `RUSTUP_TOOLCHAIN: stable` en el step de dependencias de los adapters. Verificado: `RUSTUP_TOOLCHAIN` aparece en **4** lineas de los workflows actuales. Riesgo residual: cualquier workflow nuevo que use `dtolnay/rust-toolchain` con un canal distinto al pin y luego invoque `cargo` dentro del repo reproduce el choque. | Documentar la trampa en `docs/dev/workflow/PUBLISH.md` (ya esta como "Prevencion" en el postmortem F4) y anadir `RUSTUP_TOOLCHAIN: stable` a todo workflow que use el action. | 1h |

## Veredicto

**PASS CON OBSERVACIONES**

En lo transversal, VantaDB esta bien: la superficie `unsafe` es pequena y auditada, la jerarquia de
capas es limpia, el contrato de API tiene tres capas de proteccion, y el fuzzing esta estratificado
correctamente (proptest en PR, sanitizers en Heavy Linux). Los dos hallazgos Medium tienen la misma
raiz que el postmortem de 0.9.0: **no es que falten controles, es que los controles mienten**.
El ADR Gate bloquea PRs que no deberia porque filtra por arbol de paquete en vez de superficie publica
(`FIND-405`), y la politica de publicacion documentada en `AGENTS.md` describe un mundo (publish
100 % tokenless, `gh secret list` vacio) que **dejo de ser cierto el mismo dia del release**
(`FIND-406`) - y cuya falsedad es el mecanismo exacto del fallo F1/F2. La leccion mas transferible de
esta revision esta en `W9-07`: los endpoints de lectura no prueban escrituras, y un gate que se
verifica leyendo su propia configuracion en vez de ejecutandose es un gate que no protege.

## Indice de hallazgos

| ID | Severidad | Archivo:línea |
|---|---|---|
| `FIND-405` | Medium | `.github/workflows/ci-rust.yml:167` |
| `FIND-406` | Medium | `AGENTS.md:646` |
| `FIND-407` | Critical | `SECURITY.md:26` |
| `W9-03` | Info | `src/storage/vfile_mmap.rs:1` |
| `W9-04` | Info | `.github/workflows/certification.yml:812` |
| `W9-05` | Info | `benches/scores_semantics.rs:1` |
| `W9-06` | Info | `src/error.rs:136` |
| `W9-07` | Info | `docs/dev/reviews/2026-10-20-release-v0.9.0-postmortem.md:200` |
| `W9-08` | Low | `rust-toolchain.toml:2` |
