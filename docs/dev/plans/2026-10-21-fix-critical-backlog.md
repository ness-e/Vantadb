---
title: "Plan — cerrar las 6 criticas del backlog (2026-10-21)"
kind: plan
status: active
description: "Plan de ejecucion de las 6 criticas que quedaban abiertas tras la campana del 2026-10-20/21 (FIND-518, FIND-552, FIND-553+554, FIND-343, FIND-517), agrupadas en 5 tasks por area para que ningun par toque los mismos ficheros. Cada task tiene su task file canonico con el contrato verificable."
tags: [vantadb, plan, backlog, criticas, parser, planner, engine]
---

# Plan — cerrar las 6 criticas del backlog

> **Origen:** `docs/dev/Backlog.md` (las 6 filas con severidad 🔴), tras las 4 auditorias de hueco
> de cobertura del 2026-10-21 y el cierre de la wave W6 (cifrado en reposo).
> **Alcance:** solo las criticas. Las 42 altas y las 166 medias quedan para un plan posterior.
> **Estado:** 5 tasks, todas ⬜ PENDING. Cada una tiene su task file canonico.

## Tabla resumen

| # | Task | Fila(s) | Área | Ficheros | Agente | Estado |
|---|------|---------|------|----------|--------|--------|
| 1 | `FIND-518` | FIND-518 🔴 + FIND-519 🟠 | Parser de IQL | `src/parser/**`, `src/executor.rs` | vanta-worker | ✅ COMPLETED (8d7425ab, P2-01 APTO, a09de3e0) |
| 2 | `FIND-552` | FIND-552 🔴 | physical_plan (proyección) | `src/physical_plan/project.rs`, `src/executor.rs` | vanta-worker | ✅ COMPLETED (d18c9be2, P2-01 APTO, a09de3e0) |
| 3 | `FIND-553` | FIND-553 🔴 + FIND-554 🔴 (+FIND-560 🟠) | planner + CBO + vector | `src/planner.rs`, `src/physical_plan/vector.rs`, `src/cost_estimator.rs` | vanta-engine | ✅ COMPLETED (59ab1bdb, R1/R2 en 71ba652b; FIND-521 sigue abierta) |
| 4 | `FIND-343` | FIND-343 🔴 | Durabilidad (borrado físico) | `src/wal*.rs`, `src/backends/**`, `src/storage/archive.rs` | vanta-worker | ✅ COMPLETED (WAL: 16eafbbe; journal LSM: `FjallBackend::compact()` swap quirúrgico; pin EGO-16 en verde; P2-01 APTO 2026-10-23) |
| 5 | `FIND-517` | FIND-517 🔴 | Binding de Node (reserva) | `vantadb-node/**`, `src/index/**` | vanta-worker | ✅ COMPLETED (P2-01 APTO, a09de3e0) |

**Por qué 5 tasks y no 6**: FIND-553 y FIND-554 tocan los mismos ficheros del planner (el fix de
uno es el código del otro) ⇒ van en la misma task, en orden. FIND-518 arrastra a su hermana
FIND-519 (el mismo patrón en el parser, el mismo fix estructural).

**Por qué este orden**: la 1 y la 3 son las que mas superficie de usuario tocan (una query mal
formada que se ejecuta sin filtro, y una query vectorial que devuelve filas de otro tipo) ⇒
primero. La 2 y la 5 son de correctitud acotada. La 4 es la de mayor riesgo de formato en disco ⇒
la ultima (y con la medicion de coste que exige la Regla 9).

## Dependencias

- **Ninguna entre tasks** (los conjuntos de ficheros son disjuntos) ⇒ se pueden ejecutar en
  paralelo, **con una salvedad**: la maquina tiene el **archivo de paginacion en 8 MB** y 32 GB de
  RAM ⇒ dos compilaciones de Rust simultaneas pueden fallar con `os error 1455` ("el archivo de
  paginacion es demasiado pequeno"), que es la causa real de los `E0463` que documenta el plan de
  cifrado (no es un problema de codigo). **Regla operativa: un `cargo` a la vez**
  (`cargo -j 2`).
- La task 3 (planner) y la 1 (parser) se tocan entre si por el contrato del `WHERE` (la 1 cambia
  cuando un `WHERE` mal formado es un error; la 3 toca el CBO) ⇒ si unaPulla el `WHERE` de la
  otra, la segunda debe releer el test de la primera.

## Contratos verificables (uno por task)

| Task | Contrato |
|------|----------|
| 1 | `FROM Doc WHERE n =` devuelve **error de parseo** (no un escaneo sin filtro) y `FROM Doc WHERE a = 1, b = 2` tambien; `FROM Doc;` (con `;`) sigue funcionando. |
| 2 | `FROM Doc AS OF <t> FETCH (name)` **no** devuelve nodos con `__vanta_valid_at_ms` posterior a `<t>` (hoy los devuelve todos). |
| 3 | La misma query con y sin un filtro relacional **redundante** devuelve el **mismo** conjunto de filas; un filtro presente en los datos **nunca** desaparece del plan; el path vectorial devuelve **solo** nodos de la entidad consultada. |
| 4 | Tras `delete` + compactacion, el payload **no** es recuperable ni del WAL rotado ni del journal (con `encryption` activada: no se puede descifrar sin la DEK vigente). |
| 5 | Un store **vacio** ya no ocupa 320 MB (el RSS tras `connect()` es el de la metadata) y la reserva respeta el `memory_limit` del `ConnectOptions`. |

## Gates (los mismos para todas las tasks)

```
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
cargo fmt --check
cargo clippy -p vantadb --all-targets -- -D warnings
cargo nextest run -p vantadb --profile audit --lib
```

Mas el **Review P2-01** por task: un agente **distinto** del implementador, registrado en la
seccion `## Review` del task file. Sin ese review, la task **no** se marca COMPLETED.

## Fuera de alcance (y por que)

- **Las 42 altas y 166 medias**: otro plan (por area).
- **El PR `develop → main`**: requiere el OK del owner (AGENTS.md Regla 7).
- **La migracion in-place de bases en claro a cifradas**: ADR D7 la excluye (export + import).
- **La revision P2-01 del cifrado en reposo** (`vanta-audit` + `vanta-chaos` + `vanta-review`):
  es la puerta de cierre de la wave W6 y corre en paralelo con este plan (ficheros disjuntos:
  el path de cifrado vs el parser/planner/engine).
