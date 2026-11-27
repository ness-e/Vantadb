---
title: "Verificación de las 9 filas Parcial del lote B contra HEAD: 3 vivas, 4 parciales, 1 cambiada, 1 externa"
kind: task
status: completed
description: "Auditoría de solo lectura de 9 filas Parcial contra HEAD 7aaa6c239: 3 vivas, 4 parciales reales, 1 cambiada y 1 no verificable desde el repo. Cada veredicto con archivo:línea actual o el commit que fecha el error; las externas quedan con lo exacto que falta del owner. No se implementó ni se compiló nada."
id: PARCIALES-verificacion-B
tags: [vantadb, backlog, verificacion, medicion]
---

# Verificación de las 9 filas `🟡 Parcial` del lote B

**Alcance:** medición pura. No se modificó código, no se compiló nada, no se tocó el Backlog.

**HEAD verificado:** `7aaa6c239` (2026-11-27 11:40 −04). HEAD se movió durante la
medición (`12e6d7c31` → `7aaa6c239`); la fila `WIRE-19` se re-verificó contra el
commit nuevo y el resultado no cambió.

**Método:** cada afirmación de la columna *Descripción* se contrastó con el árbol
de `HEAD` (`rg` sobre rutas y líneas reales) y, cuando la fila citaba un commit, con
`git log -S` / `git show`. Las externas se sondearon contra el servicio real (DNS,
crates.io, Notion MCP) y lo que no se pudo sondear queda declarado como tal — no se
asume ni hecho ni pendiente.

## Veredictos

| ID | Afirmación de la fila | Veredicto | Evidencia (HEAD) | Commit que fecha el error | Qué queda |
|---|---|---|---|---|---|
| `FIND-198` | Disposición documentada; falta el runner BEAM opt-in | **VIVA** | `evals/data/README.md:33-41`: MIT verificado, 4.6 MB por conversación, «runner opcional con descarga local (documentado)». Cero BEAM en código: `evals/**/*.py` no lo menciona (los hits de `longmemeval_s_subset.json` son «beam radiation», léxico) | n/a — la fila no tiene error; nace con VER-08 | El runner opt-in. `docs/dev/plans/2026-11-26-batch-parciales-y-nuevas.md:52` lo declara «decisión de tamaño/dependencia» |
| `FIND-215` | Features opt-in existen; falta medición canonical_p99, default y doc de cuándo usar cuál | **PARCIAL-REAL** | Existen: `Cargo.toml:168-169` (features) y `:200-210` (deps); doc `docs/dev/architecture/FEATURES.md:52-53`. **Refutado «falta doc de cuándo usar cuál»**: `docs/feature-matrix.md:55,75-76` documenta precedencia por OS; `.github/workflows/release-binaries.yml:117-122` (INV-004) ya elige allocator para los binarios publicados | `5deca8c78` (2026-11-27, FIND-219) — nace **después** de la fila | (a) medición: `rg -i jemalloc docs/user/operations/BENCHMARKS.md` = 0 ; (b) decisión de `[features] default` (`Cargo.toml:156` no incluye allocator) |
| `FIND-320` | Piezas existen, sin contrato `retrieve()` único | **VIVA** | `perform_auto_recall` `vanta-memory/src/core/hooks/auto_recall.rs:391`, `_governed` `:402`, `_scored` `:424`; trust gate `InjectionPolicy::trust_class` `:184`; opt-in expuesto en `vantadb-node/src/lib.rs:1161-1176`. No hay `fn retrieve` en `vanta-memory` ni freshness/conflicto/abstención en el recall | n/a — el hueco es real | El contrato. **Corrección de inventario**: `MGR-06` (`Backlog.md:331`) y `MGR-18` (`:359`) siguen `Pendiente`, **no** son «piezas que existen»; la abstención real vive en `vantadb-mcp/src/validation.rs:433` (SCH-07), no en recall |
| `FIND-598` | +43,2 % p99 post-Paso 0; causa = envelope por nodo; falta envelope por página | **VIVA** | Envelope por nodo documentado en `src/storage/vfile.rs:115`; `read_header_encrypted` parsea en sitio `:1255`; ciphertext prestado del mapping `:1340`; `memcpy` vía `align_to` `:1451-1461`; LRU acotada `:198-205`; traits `src/index_port.rs:86-88`; `flat_threshold` `src/index/search/neighbors.rs:57`. El envelope por página **es solo diseño** (`docs/dev/reviews/2026-10-21-encryption-envelope-por-pagina.md`, indexado como «Diseño, sin implementar» en `docs/index.md:446`) | n/a — las tres copias que la fila cita como eliminadas lo están; el test del reparto sigue en `vfile.rs:2085-2119` | Envelope por página = breaking change de `VectorStoreRef`: decisión del owner, no del implementador |
| `WIRE-19` | 3 de 9 hechas por `FIND-783`; 4 hijas; desviación del ADR | **PARCIAL-REAL** | Confirmado: `runner_config.rs:126,143,278,295`; métricas `fn ingest_toml_path`=1, `join("vanta-ingest.toml")`=1, `env::var("VANTADB_INGEST_CONFIG")`=1 (`:145`); shims `vantadb-server/src/scheduler.rs:63,66,75`; `read_only` `vantadb-mcp/src/server.rs:1063,1119`. **Delta de hoy**: `WIRE-20` **cerrada** ⇒ `_opt` ya existe (`vanta-memory/src/services/scheduler.rs:92`) y el bloque de intervalo ya no está en el server (`:51-54` re-export). `WIRE-21/22/23` siguen abiertas (sin task file, filas presentes) | `ddf64df2c` + `858c8361c` (2026-11-27), posteriores a la fila | WIRE-21: `runner_factory` **no existe** (`rg "fn runner_factory"` = 0 en `runner_config.rs`) y la degradación P4 vive dos veces (`scheduler.rs:78-81` + `memory_scheduler.rs:99-103`). WIRE-22: las definiciones siguen en el server. WIRE-23: `memory_scheduler.rs:69` sigue siendo el segundo read site. `CONFIGURATION.md:136,137` sin actualizar. Desviación `from_storage` **confirmada** (`init.rs:24-31` + `config.rs:1442-1447`; ADR-0063:273,400,460 sigue prescribiendo `storage.config().storage_path`) y la enmienda del ADR **no** entró: `7aaa6c239` solo corrigió la ruta falsa |
| `DIST-18` | «Publicar `vanta-proxy` (post-1.0.0)», estado `Parcial` | **CAMBIADA** | No es trabajo a medias, es un gate de versión completo: `vanta-proxy/Cargo.toml:7` `publish = false`; miembro `Cargo.toml:797`, fuera de `default-members` (`:805`, `CATEGORY: EXPERIMENTAL` `:812`); versión de workspace `Cargo.toml:820` = **0.9.0**; el crate **no existe** en crates.io (`/api/v1/crates/vanta-proxy` → 404, hoy) | Freeze desde `becaa528c` (2026-08-21) + decisión owner 2026-10-01 | Nada accionable hasta 1.0.0. Contrato correcto: «bloqueada por versión», no «parcial» |
| `GTM-05` | Landing en Vercel (`vantadb.dev`) con waitlist doble y pasarelas configuradas | **PARCIAL-REAL** | Spec y contrato de UI en repo: `docs/dev/strategy/VANTADB_DEV_LANDING_AND_ENTERPRISE_SPEC.md:137-160` (tracks `v1`/`teamsync`, `POST /api/waitlist`). **`vantadb.dev` no resuelve**: DNS NXDOMAIN y conexión «Host desconocido», medido hoy — coherente con `BND-07` «sin DNS» (`Backlog-negocio.md:66`) | n/a — el bloqueo es externo | DNS (BND-07), proyecto Vercel y pasarelas: no verificable desde el repo |
| `GTM-09` | Página `/enterprise` y one-pager OEM `Q15`+`Q16` | **PARCIAL-REAL** | One-pager **existe** y lleva el precio: `docs/dev/strategy/OEM_ONE_PAGER.md:86` («Starting at $750 / month»). Ruta `/enterprise` **especificada** (`VANTADB_DEV_LANDING_AND_ENTERPRISE_SPEC.md:78-80`) pero el dominio no resuelve (NXDOMAIN) | n/a — el bloqueo es externo | Deploy de la ruta + decisión de precio público |
| `NOT-06` | 71 páginas canónicas restauradas y pobladas, 71/71 verificadas en vivo | **NO-VERIFICABLE-DESDE-EL-REPO** | En vivo (Notion MCP, hoy): «VantaDB Docs» `3d4d0445975680f2a5fce53ad196a8ba` existe, **no** está en papelera, con **8 secciones** y **38 hijas de profundidad 2** (4+17+3+3+1+5+4+1); búsqueda `v0.6.1` = 0 resultados y `v0.7.0` solo fuera del árbol Docs. **El fichero de evidencia que la fila cita (`auditoria_pre_ejecucion_notion_cerebro_y_cierre.md`) no existe en el repo**; el plan archivado afirma 100 % (`docs/dev/plans/archive/2026-11-03-master-execution-plan.md:617-634`) | n/a | Walk de las 71 con el DoD del propio plan (0 vacías, 0 refs `v0.6.1`) y el artefacto guardado en el repo |

## Requiere al owner

1. **`GTM-05` + `GTM-09` + `BND-07` — DNS de `vantadb.dev` (un solo bloqueo para tres filas).**
   Hoy el dominio no existe. No es un problema de código: es registrar/delegar el
   dominio y apuntarlo a Vercel. **Recomendado:** resolver DNS primero; sin él, las
   otras dos no tienen dónde aterrizar y cualquier trabajo de landing es un artefacto
   local.
2. **`GTM-05` — estado de la cuenta Vercel y de las pasarelas.** No verificable desde
   el repo: hay que mirar el dashboard del proyecto en Vercel (deploy activo, dominio
   asignado) y el proveedor de pagos (¿Stripe? ¿solo links?). **Recomendado:** mantener
   las pasarelas fuera del alcance de la página hasta que exista un checkout real;
   el spec ya define la waitlist sin pago.
3. **`GTM-09` — precio público de `$750/mes`.** `OEM_ONE_PAGER.md:86` ya lo dice, pero
   es un documento interno. **Recomendado:** publicarlo tal cual en `/enterprise`
   (`start at $750`, escalando a ~$1,000): ya está escrito y coincide con
   `Backlog-negocio.md:96`; ocultarlo obliga a mantener dos discursos.
4. **`NOT-06` — acceso al workspace de Notion y criterio de aceptación.** Falta
   decidir si el «100 % completado» del plan del 2026-11-03 se acepta como evidencia
   o se exige el walk de las 71 páginas. **Recomendado:** exigir el walk con el DoD del
   propio plan y guardar el informe en `docs/dev/reviews/` — hoy la fila apunta a un
   fichero de evidencia que no existe, así que «verificado» no significa nada.
5. **`DIST-18` — no hay decisión pendiente hoy.** El freeze está completo y el crate
   no está publicado (404). **Recomendado:** re-etiquetar como «bloqueada por 1.0.0» y
   retirarla del recuento de parciales; el día del 1.0.0 sí hará falta autorización
   explícita de publish.
6. **`FIND-215` — ¿allocator en `[features] default`?** **Recomendado: no.** Los
   binarios publicados ya eligen por OS (`release-binaries.yml:117-122`), y meterlo en
   `default` obliga a cada consumidor de la librería a compilar mimalloc/jemalloc.
   Lo que falta de verdad es la medición (Regla 9), no la política.
7. **`FIND-198` — ¿se acepta un runner BEAM opt-in con descarga local?** **Recomendado:
   sí, con la disposición ya escrita** (`evals/data/README.md:33-41`): nunca en CI,
   dataset fuera del repo. Lo que hay que decidir es quién lo posee (`vanta-tuner`,
   según la propia nota).
8. **`FIND-320` — ADR del contrato `retrieve()` antes de implementar.** **Recomendado:**
   ADR corto que fije el wire shape (trust + confidence + freshness + provenance +
   conflict + abstain) y el opt-in byte-idéntico por default (precedente MEMG-21).
   Sin eso, el implementador inventará el contrato.
9. **`FIND-598` — ¿se asume el breaking change de `VectorStoreRef`?** El envelope por
   página devuelve el zero-copy pero cambia un trait público. **Recomendado:**
   post-1.0.0; hoy la fila ya tiene el número y el diseño, y el `p99` en caliente
   (+30,7 %) no cierra el gate igual.
10. **`WIRE-19` — enmienda del ADR D4 (loop OFF con la variable sin definir vs ON a
    5 000 ms).** **Recomendado:** enmendar el ADR a lo enviado (ON 5 000 en el host
    MCP) en vez de revertir: desactivar una capacidad ya enviada no es un refactor, y
    los otros tres slices (WIRE-21/22/23) cuelgan de esa decisión.

## Lo que sorprendió

- **`FIND-215` tenía dos de sus tres «faltantes» ya resueltos.** El doc de cuándo usar
  cada allocator existe desde hoy (`docs/feature-matrix.md`, `5deca8c78`) y la política
  de los binarios publicados está en `release-binaries.yml:117-122`. La fila nace
  *antes* de ese commit: el único hueco real es la medición, no las features ni la doc.
- **`NOT-06` se contradice con su propio plan.** El plan archivado del 2026-11-03
  declara «100 % COMPLETADO EN VIVO» y la fila sigue en `Parcial`, apuntando a un
  fichero de evidencia que **no existe** en el repo. Además, en vivo el árbol existe
  (8 secciones, 38 hijas) pero «71/71» no se puede afirmar sin el walk: la fila pide
  exactamente el trabajo que dice tener hecho.
- **`WIRE-19` envejeció en horas, no en días.** Una de sus cuatro hijas (`WIRE-20`) se
  cerró el mismo 2026-11-27 y dejó obsoletas tres afirmaciones de la madre: el bloque
  de intervalo, la inexistencia de `_opt` y las líneas del server (`:82,85,94` ahora
  son `:63,66,75`). La desviación del ADR, en cambio, sigue viva y sin enmienda.
- **`DIST-18` no está «parcial»: está completa.** `publish = false`, fuera de
  `default-members` y 0.9.0 en el workspace. El «Parcial» describe un trabajo que no
  empezó porque no debe empezar todavía.
- **`FIND-320` lista como existentes dos piezas que están `Pendiente`.** `MGR-06` y
  `MGR-18` figuran abiertas en el Backlog; la abstención que sí existe es del MCP
  (`validation.rs:433`), otra capa. El hueco es real, el inventario no.
- **`FIND-598` es la fila más honesta de las nueve.** Sus tres afirmaciones de mecanismo
  (ciphertext prestado, cabecera en sitio, LRU) se sostienen línea por línea, y el
  «falta el envelope por página» se confirma contra el diseño, que sigue sin implementar.
- **`GTM-09` tenía hecho lo que decía tener.** El precio exacto ya está en el one-pager;
  lo que falta no es contenido, es un dominio.

## Resumen contable

| Veredicto | Filas | IDs |
|---|---:|---|
| **VIVA** | 3 | `FIND-198`, `FIND-320`, `FIND-598` |
| **PARCIAL-REAL** | 4 | `FIND-215`, `WIRE-19`, `GTM-05`, `GTM-09` |
| **CAMBIADA** | 1 | `DIST-18` |
| **NO-VERIFICABLE-DESDE-EL-REPO** | 1 | `NOT-06` |
| **REFUTADA** | 0 | — |
| **Total** | **9** | |

**Sub-hallazgo** en `WIRE-19`: 1 de sus 4 hijas (`WIRE-20`) ya está cerrada en `HEAD`;
quedan `WIRE-21`, `WIRE-22` y `WIRE-23` abiertas.
