---
title: "Deep audit W10 - Adversarial verification of Critical/High findings"
kind: review
status: active
description: "Verificacion adversarial de los 8 hallazgos Critical/High de W0-W9: intento de refutacion de cada uno contra el codigo, con resultado (CONFIRMADO / REFUTADO / PARCIAL) y el hallazgo ajustado."
tags: [audit, reviews, verification]
---

# Deep audit W10 - Verificacion adversarial de Critical/High

> **Wave:** W10 · **Fecha:** 2026-10-20 · **Modo:** read-only
> **Especificacion:** `docs/dev/plans/2026-10-20-revision-profunda-integral-vantadb.md` §4 (W10)
> **Objetivo:** validar adversarialmente los hallazgos Critical/High de W0-W9 contra el codigo,
> descartar falsos positivos y detectar duplicados.

## Scope

Los **8** hallazgos Critical/High producidos por W0-W9, deduplicados:

| # | ID | Sev | Wave | Titulo |
|---|---|---|---|---|
| 1 | `FIND-407` | Critical | W9 | `SECURITY.md` publica una garantia de cifrado en reposo que el producto no ofrece por defecto ni sobre el WAL |
| 2 | `FIND-382` | High | W4 | Licencia de `embeddinggemma-2` mal declarada ("Gemma" vs Apache-2.0) |
| 3 | `FIND-383` | High | W4 | La descarga de `embeddinggemma-2` arrastra 5 888 MB de los que se usan 1 034 MB |
| 4 | `FIND-385` | High | W5 | 4 badges de CI apuntan a workflows inexistentes |
| 5 | `FIND-386` | High | W5 | `SECURITY.md` declara 0.5.x como version activa (el crate esta en 0.9.0) |
| 6 | `FIND-387` | High | W5 | `docs/api/scores.md` tiene 7 citas `archivo:línea` obsoletas |
| 7 | `W3-01` | High | W3 | 2 de las 49 tools MCP no son alcanzables desde la skill de usuario |
| 8 | `FIND-390` | High | W8 | 11 `continue-on-error: true` sin el tag `# CATEGORY:` |

**Excluido con motivo:** los Medium/Low/Info (no pasan el umbral de "bloquea un release" y ya estan
en el backlog con su severidad). `FIND-343`/`FIND-249` (borrado fisico) ya estaban registrados
previamente - **no se re-ticketan** (verificacion explicita en W6-03).

## Metodologia

Para cada hallazgo: (1) buscar evidencia que lo **refute**; (2) si no la hay, re-verificar la
evidencia original con un metodo **distinto** al usado en la wave original; (3) registrar el
resultado y, si el hallazgo resulta overstated, **ajustarlo** en el reporte de origen.

## Verificacion adversarial

| # | ID | Intento de refutacion | Resultado | Veredicto |
|---|---|---|---|---|
| 1 | `FIND-407` | *¿Estara `encryption` activo en el crate publicado, lo que haria la claim de `SECURITY.md` correcta?* Se busco `encryption` en los `Cargo.toml` de todos los artefactos distribuibles: `vantadb-server/Cargo.toml` -> `encryption = ["vantadb/encryption"]` (opt-in); `vanta-proxy/Cargo.toml` -> `vantadb = { path = "..", default-features = false, features = ["encryption"] }` (**activado**); `vanta-memory/Cargo.toml` -> `erasure = ["vantadb/encryption", ...]` (opt-in); `vantadb-wasm/Cargo.toml` -> `default = ["tracing-wasm"]` (**sin** encryption); `vantadb-python/Cargo.toml` y `vantadb-node/Cargo.toml` -> sin `features` de encryption. Y `Cargo.toml` raiz: `default = ["cli","arrow","fjall","roaring","advanced-tokenizer","memmap2","fs2","sysinfo","rayon"]` - `encryption` ausente. | **REFUTADO PARCIALMENTE.** El crate `vantadb` de crates.io (el producto principal) se publica **sin** cifrado, y `vanta-wasm` tambien. `vanta-proxy` si lo tiene. Ademas, el hallazgo original se mantiene integro en su parte mas grave: la busqueda de `Cipher|EncryptionStream|encrypt` en `src/wal.rs` y `src/wal_sharded.rs` da **0** - **la WAL no se cifra en ningun canal**, con o sin la feature. | **PARCIALMENTE CONFIRMADO.** Hallazgo ajustado en W9 (nota de refutacion anadida al titulo y a la descripcion). Se mantiene Critical por dos razones independientes: (a) el crate principal publicado no tiene el cifrado que la politica de seguridad declara como propiedad del producto; (b) la WAL queda en claro en todos los canales, lo que hace que "at-rest encryption" sea una garantia parcial presentada como total. |
| 2 | `FIND-382` | *¿ sera la licencia "Gemma" correcta y el manifest el equivocado?* Se consulto la **API de Hugging Face** (metadatos de la ficha, no la pagina renderizada): `GET https://huggingface.co/api/models/google/embeddinggemma-2` -> `cardData.license = "apache-2.0"`, `tags = ["license:apache-2.0", ...]`. Idem `GET https://huggingface.co/api/models/onnx-community/embeddinggemma-2-ONNX` -> `cardData.license = "apache-2.0"`. Y el model card oficial en `https://gemma.ai/models/embeddinggemma/model-card-v2/` -> "License : Apache 2.0". | Refutacion fallida por **tres** fuentes independientes, incluida la API. | **CONFIRMADO** (y reforzado: la fuente estructurada coincide con la documentada). El manifest es el unico que discrepa. |
| 3 | `FIND-383` | *¿Seran 5 888 MB un artefacto local y no el comportamiento de `download.py`?* Se leyo el codigo: `download.py:30` `ALLOW_PATTERNS = [..., "onnx/*", ...]`; `download.py:34` `MODEL_PATTERNS: dict[str, list[str]] = {}` (**vacio** -> todos los modelos usan el glob). `download.py:219-223` llama `snapshot_download(..., allow_patterns=get_allow_patterns(mid))`. Con `onnx/*` sobre un repo multimodal, `snapshot_download` trae **todo** `onnx/`. La medicion de disco (`embeddings/models/embeddinggemma-2/onnx/`: `model.onnx_data` 1 033.9 MB + `audio_encoder.onnx_data` 1 118.4 MB + `vision_encoder.onnx_data` 639.9 MB + 12 variantes cuantizadas) es exactamente la firma que produce `onnx/*`. | Refutacion fallida: el patron es el vigente y la medicion lo confirma. | **CONFIRMADO.** |
| 4 | `FIND-385` | *¿Habra workflows con esos nombres que no esten trackeados pero existan en GitHub?* `git ls-files .github/workflows` devuelve exactamente **20** ficheros: `certification, ci-fleet, ci-gate, ci-rust, demo-acceptance, docs-gates, electron-node, nightly, perf-bench, platform-ops, publish-mcp-registry, release-adapters, release-binaries, release-npm-61, release-npm-node, release-providers, release-sbom, release-verify, release-wheels, release`. Ni `gate-docs.yml` ni `sec-codeql.yml`. Ademas los headers de los workflows reales lo confirman: `docs-gates.yml:4` "Origenes: gate-docs.yml, ..." y `platform-ops.yml:4` "Origenes: desktop.yml, sec-codeql.yml, ...". | Refutacion fallida en las dos vias (filesystem y git). | **CONFIRMADO.** El badge de "Security Audit" quedara permanentemente en "unknown" porque el workflow se renombro a `platform-ops.yml` en el PR de consolidacion SRV-10. |
| 5 | `FIND-386` | *¿Estara "0.5.x" correcto por algun motivo (p.ej. la linea commercially supported)?* `git log --oneline -3 -- SECURITY.md` -> el ultimo cambio es `58a41ad8 release: develop -> main (0.6.1 + estabilizacion + docs/user-dev) (#222)`. Es decir, el fichero **no se toca desde la release 0.6.1**. No hay ninguna nota, tabla o convencion que limite el alcance de "Supported Versions" a una linea comercial concreta. | Refutacion fallida; se identifica la **causa raiz**: el documento quedo congelado en la release 0.6.1. | **CONFIRMADO**, con causa raiz identificada. |
| 6 | `FIND-387` | *¿Seran las citas de `scores.md` correctas y mi lectura de las lineas equivocada?* Se re-verificaron **una por una** las 7 citas con `Read` sobre la linea exacta (no con grep de contexto): `src/planner.rs:27` = doc-comment de "Selectivity threshold"; `:96` = `arms: Vec<Box<dyn crate::query::PhysicalOperator + 'a>>,`; `:129` = `arm.close()?;`; `:205` = `*right_plan.clone(),`; `:221` = `} => {`; `src/sdk/search/mod.rs:108-120` = doc-comment de ejemplo. Se localizaron los destinos reales: `RRF_K` -> `src/search_profile.rs:20`; `hybrid_candidate_budget` -> `src/sdk/search/fusion.rs:23`; zero-norm -> `src/sdk/search/mod.rs:244`. Y se verificaron las 3 rutas inexistentes con `Get-ChildItem -Recurse -Filter`: `integrations/langchain/vectorstore.py` -> real `integrations/langchain/vantadb_langchain/vectorstore.py`; `integrations/llamaindex/vectorstore.py` -> real `integrations/llamaindex/vantadb_llamaindex/vectorstore.py`; `desktop/retrieval-core.ts` -> real `feature-freez/desktop/src/components/lens/retrieval/retrieval-core.ts`. | Refutacion fallida en las 10 referencias. Se re-verifico además que `src/sdk/search/debug.rs:23` (=`rank_map`) y `src/index/distance/metrics.rs:47` (=`cosine_sim_f32`) **si** son correctas - el documento esta parcialmente bien, lo que confirma que se trata de deriva de lineas, no de un documento inventado. | **CONFIRMADO.** |
| 7 | `W3-01` | *¿Estaran las 2 tools documentadas con otro nombre o en otra parte del SKILL.md?* Diff programatico: se extrajeron los 49 nombres de `"name": "<tool>"` de `vantadb-mcp/src/handlers/tools.rs:82` (`handle_tools_list`) y se comprobo la presencia literal de cada uno en el texto de `skills/vantadb-mcp/SKILL.md`. Resultado: 47 presentes, 2 ausentes: **`memory_reinforce`** y **`memory_verify_certificate`**. Se descarto el falso positivo del patron "alias legacy" comprobando que `collection_list` si aparece mapeado en `tools.rs:1319` (`"collection_list" => Some("memory_list_namespaces")`). | Refutacion fallida. | **CONFIRMADO.** La ausencia de `memory_verify_certificate` en la skill es la mas relevante: un agente que solo siga la skill no puede verificar un certificado de borrado. |
| 8 | `FIND-390` | *¿Tienen las 11 lineas el CATEGORY en una linea anterior o en un comentario de contexto?* Se filtro `continue-on-error: true` y seRequirement cada linea que contiene `CATEGORY:`. Resultado: 27 de 38 tienen el tag en la misma linea; **11 no lo tienen en ninguna forma**: `certification.yml:534`, `ci-rust.yml:360,454,650,693,782`, `platform-ops.yml:221`, `release-adapters.yml:167`, `release-npm-node.yml:244`, `release-providers.yml:267`, `release-wheels.yml:197`. | Refutacion fallida. | **CONFIRMADO.** 4 de las 11 estan en canales de publicacion de artefactos. |

## Analisis de duplicados

| ID candidato | ¿Duplicado de algo existente? | Veredicto |
|---|---|---|
| `FIND-407` (cifrado) | `docs/dev/reviews/2026-10-20-release-v0.9.0-postmortem.md:222` lo lista como "P0 de `SECURITY.md` (falsa garantia de cifrado en reposo) ... solo se puede corregir en 0.9.1" | **No es un duplicado**: el postmortem lo lista como pendiente de un incidente, **sin fila de backlog**. Se emite como `FIND-407` con `Relaciones: Origen: ...postmortem.md`. |
| `FIND-405` (ADR gate) | Postmortem "Pendiente de este incidente": "Filtro del **ADR Gate**: que mire la superficie publica real, no `vantadb-ts/`" | Idem: pendiente sin ticket. Se emite como `FIND-405`. |
| `FIND-406` (politica de publish) | `AGENTS.md:646-648` es el propio doc a corregir; el postmortem F1/F2 es la evidencia | No duplicado. |
| `FIND-385` / `FIND-399` (badges / refs stale) | `FIND-204` (Baja) cubre solo `CONTRIBUTING.md` + `.config/nextest.toml` para `heavy-certification-50` y quedo "Parcial - sweep incompleto" | **No duplicado**: `FIND-385` es el badge (superficie visible, no cubierta) y `FIND-399` **amplia** `FIND-204` a 100 refs en 24 ficheros. Se referencia `FIND-204` en `Relaciones`. |
| `FIND-343` / `FIND-249` (borrado fisico) | Ya registrados (Alta / Media) | **Re-ticket NO emitido.** W6-03 los referencia. |

**Sin hallazgos duplicados entre waves**: los 8 Critical/High son ortogonales (1 doc-compliance,
2 licence/cost, 2 docs-de-la-raiz, 1 doc-tecnico, 1 superficie MCP, 1 CI). El unico cruce posible
era `FIND-407` (W9) y `FIND-386` (W5), ambos sobre `SECURITY.md` pero sobre **lineas distintas**
(26 vs 5-9) y con causas distintas: garantia falsa vs politica de soporte desactualizada. Se mantienen
separados por legibilidad del fix, y se cruzan en `Relaciones`.

## Veredicto

**PASS**

Los **8** hallazgos Critical/High sobreviven el intento adversarial. Siete se confirman sin fisuras
(incluida la confirmacion de `FIND-382` por la **API** de Hugging Face, que es evidencia
estructurada y no una pagina renderizada). Uno (`FIND-407`) se **refuta parcialmente** y se ha
**ajustado** en su reporte de origen (W9) con la precision que faltaba: `vanta-proxy` si activa el
cifrado, pero el crate `vantadb` publicado no, y la WAL no se cifra en **ningun** canal - que es la
parte del hallazgo que sostiene la severidad Critical. No se descarto ningun hallazgo por ser falso
positivo, y se detecto y neutralizo un intento de re-ticketear el borrado fisico, que ya estaba
registrado como `FIND-343`/`FIND-249`.

## Indice de hallazgos

| ID | Severidad | Archivo:línea |
|---|---|---|
| `W10-01` | Info | `SECURITY.md:26` |
| `W10-02` | Info | `embeddings/manifest.json:129` |
| `W10-03` | Info | `embeddings/download.py:30` |
| `W10-04` | Info | `README.md:9` |
| `W10-05` | Info | `SECURITY.md:5` |
| `W10-06` | Info | `docs/api/scores.md:22` |
| `W10-07` | Info | `vantadb-mcp/src/handlers/tools.rs:82` |
| `W10-08` | Info | `.github/workflows/release-wheels.yml:197` |
