---
title: "FIND-733 — ADR gate: separar superficie real (parse_iql) de items fantasma (impl de tipos privados)"
kind: task
status: completed
description: "El ADR Gate ve +3 items en el push b7a5d2eab; se audita EmbedCache/WarmLocal (ya privados, sin pub que degradar) y se documenta parse_iql en ADR-0064"
---

# FIND-733 — ADR Gate: superficie real vs. items fantasma

> **Plan:** `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md` (desbloquea el PR `develop → main`)
> **Ruta:** `vanta-arch` · **Carril:** A/R · **Appetite:** 2-3h
> **Dependencias:** `FIND-706` ✅ (`82077466c`, creó los tipos) · `FIND-722` ✅ (`c8d9e95a`, creó `parse_iql`) · `FIND-731` (falso verde del golden, registrado en `b7a5d2eab`)

## Contexto

El monitor de CI reportó que el PR `develop → main` va a fallar el `ADR Gate`
porque el push `b7a5d2eab` añadió 3 items de superficie pública sin ADR. La
evidencia medida:

```
$ node scripts/docs/check-api-docs.mjs --changed 06485fe8e..b7a5d2eab
rust surface changed: +3 -0 (1715 -> 1718 items)
  + rust fn executor::parse_iql          <- FIND-722, intencional
  + rust impl llm::EmbedCache            <- FIND-706, ¿intencional?
  + rust impl llm::WarmLocal             <- FIND-706, ¿intencional?
```

La pregunta de la tarea: ¿`EmbedCache` y `WarmLocal` se filtraron a la API
pública, o son estado interno del provider?

## Auditoría (parte a) — resultado medido

**No se filtraron, y no hay `pub` que degradar.** Los dos tipos ya son privados
en `HEAD`:

| Tipo | Declaración real | ¿Público? |
|---|---|---|
| `EmbedCache` | `src/llm.rs:239-243` → `#[derive(Default)] struct EmbedCache` | **no** (sin `pub`) |
| `WarmLocal` | `src/llm.rs:266-284` → `struct WarmLocal` | **no** (sin `pub`) |
| `WARM_LOCAL` (el `static` que los guarda) | `src/llm.rs:210-212` → `static WARM_LOCAL` | **no** |

Tres verificaciones independientes, no una:

1. **`cargo public-api` (el compilador) nunca los listó.** El golden
   `tests/api/public-api.txt` tiene **una sola** línea de `llm` que no sea
   `config::LlmCfg`: ninguna de los dos tipos. Delta real del golden en el rango
   `06485fe8e..HEAD`: **+1 línea**, sólo `parse_iql`.
2. **Nadie fuera de `src/llm.rs` los nombra.** `rg -n 'EmbedCache|WarmLocal'
   --glob '*.rs' -l` → `src\llm.rs` (único archivo; incluye el test unitario
   `llm.rs:2126`, que es interno).
3. **El consumidor no los necesita.** El contrato público es
   `EmbeddingProvider::embed/embed_query/embed_batch/is_synthetic` +
   `LocalOnnxProvider::{new, new_dummy}` + `is_real`. La caché y la sesión
   caliente son estado del provider; exponerlas invitaría a depender de la
   política de evicción (FIFO, 256 entradas) y del ciclo de vida del `Session` de
   ONNX — Hyrum's Law al revés: contrato inventado sin consumidor.

**Por qué el gate los ve igual:** `extractRust()`
(`scripts/docs/check-api-docs.mjs:357-374`) registra `rust\timpl\t<Tipo>` para
**cada** bloque `impl` de nivel superior y **no consulta visibilidad**. Un
`struct` privado con `impl` produce un item de superficie que no existe. Medido:
46 de los 176 `impl` del crate corresponden a tipos que no son `pub` en ninguna
parte (guards, mocks de test, `MockScan`, estos dos…), así que el gate
sobre-reporta de forma estructural, no solo aquí.

**Conclusión:** `src/llm.rs` **no se toca**. No hay fuga de visibilidad; hay un
falso positivo del extractor. El único cambio de superficie real es `parse_iql`
(+1) → parte b.

## Impacto mapeado (Regla 0)

**Leídos completos antes de editar**

| Archivo | Qué se leyó | Por qué |
|---|---|---|
| `src/llm.rs:1-1217` | todo el módulo: `EmbeddingProvider`, factorías, `WarmLocal`/`EmbedCache`, `embed_as` | es el objeto de la auditoría (a) |
| `scripts/docs/check-api-docs.mjs:1-1330` | extractores + `evaluate()` + self-test | explica por qué el gate ve `impl` privados |
| `.github/workflows/ci-rust.yml:228-290` | job `adr-gate` completo | la regla es **por path**, no por superficie |
| `docs/dev/tasks/FIND-722.md:1-162` | decisión, 8 sub-decisiones, delta del golden, review | el ADR documenta lo ya decidido, no lo re-decide |
| `docs/dev/architecture/adr/ADR-0062-ast-json-numeric-fidelity.md` | frontmatter, `### O<n>`, `### D<n>`, consecuencias | estilo MADR del repo |
| `docs/api/EMBEDDINGS.md:55-99` | §`LocalOnnxProvider` internals | ya documenta `WarmLocal` como interno |
| `docs/api/IQL.md` (vía `FIND-722`) | contrato de posición publicado | el ADR lo cita, no lo duplica |

**Referencias hacia dentro (¿quién los nombra?)**

- `EmbedCache` / `WarmLocal`: sólo `src/llm.rs` (`rg -l` → 1 archivo). Ni
  bindings, ni tests de integración, ni `tests/api/public_api.rs`.
- `parse_iql`: `src/executor.rs` (`execute_hybrid`), `vantadb-python/src/lib.rs`,
  `vantadb-wasm/src/lib.rs` (y TS vía WASM). Todos dentro del cambio de
  `FIND-722`, ya revisado.

**Referencias hacia fuera**

- `src/llm.rs` → `ort`, `tokenizers`, `crate::memory_governor`, `crate::config`.
  Ninguna cambia.
- ADR-0064 → `FIND-722`, `ADR-0062`, `docs/api/IQL.md`.

**Veredicto:** blast radius real = **documentación** (1 ADR nuevo + 1 fila de
índice generado + 1 aclaración en `docs/api/EMBEDDINGS.md`). Cero cambios de
código, cero cambios de golden. El gate `adr-gate` queda satisfecho por el ADR.

## Steps

1. ✅ Auditar `EmbedCache`/`WarmLocal`: privados, sin consumidores externos,
   ausentes del golden del compilador → no se toca `src/llm.rs`.
2. ✅ Verificar el delta real del golden: `git diff 06485fe8e..HEAD --
   tests/api/public-api.txt` → **+1** (`parse_iql`), no +3.
3. ✅ Escribir `ADR-0064` (MADR, 5 opciones, 8 sub-decisiones) documentando la
   decisión ya tomada por `FIND-722`.
4. ✅ Aclarar en `docs/api/EMBEDDINGS.md` que `WarmLocal`/`EmbedCache` son tipos
   privados, fuera del contrato (evita que el próximo lector lea el `+2` del
   gate como superficie).
5. ✅ Regenerar índices derivados (`gen-index.mjs --write`) para publicar el ADR.
6. ✅ Contrato mecánico + review P2-01 + commit local.

## Contrato

Hay **dos gates reales** y dos comandos de diagnóstico. La distinción importa:
el primero no se pone verde con este cambio, y no debe.

```powershell
# GATE 1 — ADR (por path, no por superficie): el job `adr-gate` exige un ADR en el PR
#   cuando cambió src/ o un binding. Se satisface con ADR-0064 presente.
#   Umbrella de Fase 0, no se corre acá: node scripts/docs/check-api-docs.mjs --changed <base>..HEAD

# GATE 2 — el golden del compilador (la superficie de verdad)
cargo nextest run -p vantadb --test public_api --run-ignored ignored-only --locked

# DIAGNÓSTICO — no es gate, y sigue diciendo +3 (2 son impl de tipos privados)
node scripts/docs/check-api-docs.mjs --changed 06485fe8e..HEAD
git diff 06485fe8e..HEAD -- tests/api/public-api.txt   # lo que importa: +1

# Forma del ADR
(Select-String -Path docs/dev/architecture/adr/ADR-0064-iql-parse-error-formatter.md -Pattern '^### O[0-9]+ ').Count
```

## Verificación

| Gate | Comando | Resultado |
|---|---|---|
| Superficie real | `git diff 06485fe8e..HEAD -- tests/api/public-api.txt` | **+1 línea**: `pub fn vantadb::executor::parse_iql(…)`. Ni una más |
| Golden vs código | `cargo nextest run -p vantadb --test public_api --run-ignored ignored-only --locked` | ✅ PASS (348 s; byte-idéntico, no se regeneró nada en esta tarea) |
| ADR Gate | `ci-rust.yml` job `adr-gate` (path-based) | ✅ satisfecho por `ADR-0064` en `docs/dev/architecture/adr/` |
| Diagnóstico (no gate) | `node scripts/docs/check-api-docs.mjs --changed 06485fe8e..HEAD` | `+3`: `parse_iql` + 2 `impl` de tipos privados (§Auditoría, medición 3) |
| Opciones del ADR | `(Select-String … '^### O[0-9]+ ').Count` | 5 (≥2 exigidas) |
| Docs | `check-links` → 0 broken · `check-docs` → `GATING: all clear` · `gen-index --check` → exit 0 | ✅ |
| markdownlint | `npx markdownlint-cli2 --config .github/markdownlint-cli2.yaml` (ADR) | ✅ 0 issues |
| Review P2-01 | `vanta-review` (contexto fresco ≠ autor) | ver §Review |

## Definition of Done

- Nivel **Task** (documentación + auditoría; sin código).
- [x] Auditoría con tres verificaciones independientes (§Auditoría).
- [x] ADR escrito con ≥2 opciones reales y consecuencias negativas.
- [x] Índices generados regenerados (nunca a mano).
- [x] Review P2-01 por agente distinto.
- **Release: no aplicable** — sin `feat:`/`fix:` de código; el ADR es `docs:`.

## Review

**P2-01 (ronda 1, `vanta-review`, contexto fresco ≠ autor):** `changes-required`
→ 2 Critical + 2 High + 2 Medium + 2 Low.

| # | Sev | Hallazgo | Disposición |
|---|---|---|---|
| C1 | Critical | Índices generados sin regenerar (otro carril corrió `gen-index --write` entre la escritura y la lectura) y el DoD los daba por hechos | ✅ Re-regenerados; `gen-index --check` sin bloque `stale`; ADR-0064 en `adr/README.md:83`, `docs/index.md` = 66 ADRs |
| C2 | Critical | `adr/README.md` no figuraba como modificado | ✅ Mismo origen que C1; ahora `M` real |
| H3 | High | El ADR afirmaba una fila de follow-up "registered" que no existía | ✅ "returned for **registration**… **not yet in the Backlog**; no ID claimed" |
| M1 | Medium | Refs de línea obsoletas (copiadas de `FIND-722`, pre-refactor) | ✅ `:90` / `:120` / `:357` + `parse_iql:217`, verificados contra `src/executor.rs` |
| M2 | Medium | §D8 describía el trigger del gate como "any change under `src/`" | ✅ Los 5 paths del `for p in …` + la debilidad real del `[no-adr]` (`contains()`, `::warning::`) |
| L1 | Low | §Contrato mezclaba gates reales con diagnósticos | ✅ GATE 1 / GATE 2 / DIAGNÓSTICO separados en prosa y en tabla |
| L2 | Low | Cifra 46/176 sin método declarado y fila sin ID | ✅ Medición declarada (ni `pub` ni re-exportado) + "todavía no tiene ID" |

**P2-01 (ronda 2, mismo reviewer, verificación de disposiciones): `approve`.**
Los 7 cerrados con evidencia; sin errores factuales introducidos por las
correcciones. Caveat escrito: el numerador 46 es consistente con el extractor
pero no recontado tipo por tipo — es diagnóstico, no gate. Residual cosmético
(`ci-rust.yml:228-283` → `:290`) corregido después del veredicto.

**Veredicto final: approve.**

## NOTICED BUT NOT TOUCHING

- **`check-api-docs.mjs` sobre-reporta:** `extractRust` (`:357-374`) registra
  `impl` de tipos privados como superficie. Medido sobre el crate completo:
  **46 de los 176** items `impl` corresponden a tipos que no están declarados
  `pub` en ningún sitio ni re-exportados públicamente (`EmbedCache`,
  `WarmLocal`, `EnvGuard`, `MockScan`, `FakePorts`, …). Arreglo correcto: exigir
  que el tipo haya sido declarado `pub` (o re-exportado) antes de contar su
  `impl`. No se hace acá porque `scripts/docs/**` es carril C (`FIND-713`,
  `FIND-264`) y el arreglo mueve 46 entradas del conteo → necesita su propio
  contrato. **Fila devuelta al orquestador** (texto en el RESULTADO de la tarea;
  todavía no tiene ID en el Backlog).
- **`tests/api/public_api.rs:123`** usa `var_os(UPDATE_ENV).is_some()`: una
  variable presente pero vacía entra en modo update y reescribe el golden
  devolviendo PASS (`FIND-731`, falso verde registrado en `b7a5d2eab`).
  Recordatorio operativo: al regenerar, usar `VANTADB_PUBLIC_API_UPDATE=1`
  explícito y **comprobar que el golden cambió**.
- **Árbol compartido:** mientras corría esta tarea entraron `264940d57`
  (`FIND-732`) y `7e55dec22` (`FIND-735`), y otros carriles tienen WIP en
  `vanta-memory/**`, `vantadb-mcp/**` y `.github/workflows/ci-rust.yml`. Los
  índices generados son un artefacto compartido: se regeneran al commitear y
  pueden incluir filas de ficheros de otros carriles de la misma wave.
