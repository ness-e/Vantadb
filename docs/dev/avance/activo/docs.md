---
title: "Avance — Documentación de producto y gates de docs"
kind: review
status: active
description: "Registro del trabajo completado sobre la capa de documentación: gates de docs, capa product-first y tooling del corpus"
tags: [vantadb, avance, docs]
---

# Avance — Documentación de producto y gates de docs

> Registro consolidado del trabajo completado sobre `docs/`: gates de CI (links, schema, secretos, índices),
> tooling del corpus (stamper de frontmatter) y la capa de documentación orientada a producto.
> IDs originales conservados.

### Capa product-first: decisión de adopción (`FIND-323`)
- **Fecha:** 2026-11-24
- **Objetivo:** El repo tenía referencia exhaustiva pero ninguna capa de decisión: qué es VantaDB, si sirve para un caso concreto y qué runtime elegir. Curaduría sobre `PYTHON_SDK.md`, `TS_SDK.md`, `NODE_SDK.md`, `BINDINGS_NAMESPACES.md` y `ARCHITECTURE.md`, sin duplicar contenido.
- **Resultado:** ✅ Tres páginas nuevas, cada una con su `kind` derivado de la ruta:
  - `docs/user/concepts/what-is-vantadb.md` (concept): qué es en un párrafo con cada cláusula enlazada, más **9 filas de "qué NO es"**, cada una apuntando al documento que posee esa frontera.
  - `docs/user/IS_IT_FOR_ME.md` (howto): **13 casos de uso** × veredicto (built-for / fit / partial / not-a-fit) × qué obtienes × qué debes aceptar × fuente.
  - `docs/user/WHICH_RUNTIME.md` (howto): **8 runtimes** × cuándo usarlo, persistencia, capa cognitiva y referencia, más instalación por runtime y las trampas native-vs-WASM y de paridad.
  - Regla "una verdad, un fichero": **36 enlaces a `docs/api/**`, 0 firmas copiadas**; cero cifras de performance (solo enlace a `BENCHMARKS.md`, Regla 11). `docs/api/**` no se tocó.
- **Verificación:** `check-docs` 0 · `check-links` 0 · `gen-index --check` 0 · markdownlint 0 issues en las 3 páginas · orphans 0/720 · las 3 aparecen en `docs/index.md`, `docs/user/index.md` y `llms.txt`.
- **Commit:** `0b99ae80b`
- **Notas:** los gates cazaron 5 enlaces rotos (rutas `../` escritas como si el fichero estuviera en `concepts/`) y 3 afirmaciones sin fuente que se borraron antes del commit. **Incógnita para el owner:** el repo no declara un runtime oficialmente recomendado para un proyecto nuevo; la jerarquía de `WHICH_RUNTIME.md` se deriva de las tablas de los SDKs, y declarar un default es decisión de producto.

### `segment()` BOM-safe + self-test del stamper en CI (`FIND-713`)
- **Fecha:** 2026-11-24
- **Objetivo:** Cerrar el segundo lector del mismo fichero que `FIND-707` dejó pendiente: `segment()` probaba `src.startsWith("---\n")` sobre el texto crudo, así que un BOM convertía el bloque de frontmatter en `prose` (`safe:true`) y lo entregaba al único escritor de segmentos (`wikilinks-to-md.mjs`). Y cablear el `--self-test` del stamper, que solo corría a mano.
- **Resultado:** ✅ Un solo normalizador (`bomLength`, `lib.mjs:146`) usado por `parseFrontmatter` y `segment()`; el BOM se emite como segmento propio para no romper `segmentsRoundTrip`. 7.º caso del self-test (RED contra `HEAD` real → exit 1; GREEN 7/7) con 3 mutantes muertos. Step en el job `docs-schema` de `docs-gates.yml` (sin `if:`, required en `main`: bloquea `develop → main`). Gates: `check-links`/`check-docs`/`check-secrets`/`wikilinks-to-md --self-test` en 0, `--write` idempotente (794 unchanged), `actionlint` 0.
- **Commit:** `b319de9fb`
- **Hallazgo derivado (fila `FIND-719`):** **ningún job de `docs-gates` es required en `develop`** (los 10 contexts requeridos son de Rust), así que el gate de docs solo muerde en el PR de release.
- **Deuda:** `segment()` sigue sin la rama "bloque cerrado en EOF sin salto final" que `parseFrontmatter` ganó en `FIND-707` (0/795 exposición; 3 docs con `[[` en frontmatter) · `check-structure.findOrphanProse` es un 7.º lector BOM-ciego.

### El extractor de API contaba `impl` de tipos privados (`FIND-736`)
- **Fecha:** 2026-11-25
- **Objetivo:** El índice de API y el **ADR Gate** emitían `rust impl T` sin consultar si `T` era público, así que cualquier helper interno recién añadido aparecía como item de superficie y **exigía un ADR que no correspondía**. Al cerrar `FIND-733` reportó `+ rust impl llm::EmbedCache` y `+ llm::WarmLocal` sobre `struct` **sin `pub`**.
- **Resultado:** ✅ `extractRust` arma ahora el conjunto de tipos públicos **después** del walk (declaraciones `pub` en módulos recorridos + nombres de `pub use`) y filtra `impl`/`method` contra él. Matching por último segmento: un `Foo` privado puede heredar el veredicto de un `Foo` público — **falso positivo elegido a propósito** sobre el falso negativo que ocultaría un `impl` público real.
- **Delta medido:** `rust impl` **176 → 130** (−46) · `rust method` **490 → 482** (−8) · rust surface **1718 → 1664** · total 3 lenguajes **2716 → 2662** · **added 0** (puramente sustractivo). Los 8 métodos perdidos son todos de tipos `pub(crate)`: `AlignedBlob` ×4, `NodeBytes::as_slice`, `ProcessCryptoGuard::acquire`, `WalVerifyStatus::{as_str,is_ok}`. Familias de los 46: guards (9), dobles de test `Mock*`/`Fake*`/`Passthrough*` (12), servicios E2E de `server::bootstrap` (6), helpers privados del motor (13), artefactos no-tipo (5).
- **Efecto sobre el ADR Gate:** (1) añadir un `impl` a un tipo privado/`pub(crate)` **ya no** cambia el conteo → no exige ADR; `--changed 06485fe8e..HEAD` pasó de **+3 a +1** (`executor::parse_iql`) y los dos fantasmas de `FIND-733` desaparecen. (2) Sin cambios para `impl` de tipos `pub` o nombrados por `pub use`. (3) **Mejora no buscada: ampliar visibilidad ahora sí produce delta** (`struct Foo` → `pub struct Foo` genera `+ rust impl Foo`), así que el gate ve antes lo que no veía. (4) Caso que podría ocultar, declarado con **0 instancias medidas**: un `impl` escrito a mano sobre un `pub struct` generado por macro; el golden del compilador lo cubriría por otra vía.
- **Verificación:** `--self-test` **19/19** · `--api-surface` con el delta pegado · `rg 'impl .*::(EmbedCache|WarmLocal)' docs/api/` = 0 · review P2-01 ronda 1 `changes-required` (F1/F3/F4/F6 corregidos; F2/F5/F8 documentados con evidencia; F7 rechazado por Regla 7) → ronda 2.
- **Commit:** `459ff361a`
- **Nota de aterrizaje:** `docs/index.md` de su commit incluye las filas de `MCP-44` y `MEM-71` (task files de otros carriles, aún sin trackear): en un checkout aislado son enlaces colgantes hasta que aterricen, y `gen-index --write` se corre en el cierre de wave.
