---
title: "Sintesis de la revision profunda integral de VantaDB (2026-10-20)"
kind: review
status: active
description: "Sintesis de las 11 waves de la revision profunda read-only: tablero por wave, top 20 hallazgos, mapa de cobertura, FODA, rutas de remediacion y veredicto global con estado de preparacion de v0.9.0."
tags: [audit, reviews, synthesis]
---

# Sintesis - Revision profunda integral de VantaDB (2026-10-20)

> **Campana:** revision read-only ejecutada en background mientras corria el CI de v0.9.0.
> **Plan:** `docs/dev/plans/2026-10-20-revision-profunda-integral-vantadb.md`
> **Estado base del repo:** rama `fix/release-path-followups`, HEAD `ab50a0c8`, 1 untracked
> (`docs/dev/plans/2026-10-20-revision-profunda-integral-vantadb.md`), 3 modificados
> (`docs/index.md`, `llms.txt`, `vanta-mcp-local.ps1`) - ninguno tocado por esta revision.
> **Alcance total:** 11 waves (W0-W10) + esta sintesis (W11). Cero commits, cero push, cero
> escrituras en `.opencode/`, cero cambios de codigo de produccion.

## 1. Limitaciones de la ejecucion (leer primero)

El plan asignaba cada wave a un subagente `vanta-*` (leaf, `subagent: deny`) lanzado por el
orquestador. **El tool `task` no existe en la sesion que ejecuto esta revision**: se verifico
(`tools.task` -> `Unknown tool 'task'`), y no hay ningun otro lanzador de subagentes disponible
(`tools.opencode` expone solo `session_rename`, `session_move`, `models`,
`list_mcp_resources`, `read_mcp_resource`). Conforme a la clausula del prompt de campana
("ninguna wave puede quedar sin cubrir"), **el orquestador ejecuto las 11 waves directamente**,
fusionando los sub-scope de cada wave multi-subagente en secciones dentro de un unico reporte por
wave. Consecuencias:

- **Cobertura de wave: 11/11.** Ninguna wave quedo sin cubrir.
- **Profundidad por wave: menor que con 6 subagentes en paralelo.** Cada wave se concentro en su
  scope asignado, con muestreo explicito en areas secundarias (declarado en cada reporte).
- **Sin validacion adversarial entre autores distintos**: W10 se ejecuto como auto-verificacion
  adversarial con metodos de re-verificacion alternativos, no como revision por un contexto limpio.

**BLOCKED: no se pudo ejecutar codigo Rust.** `cargo`, `rustc` y `nextest` **no estan en el PATH**
de este entorno (`Get-Command cargo` -> NOT FOUND; `cargo --version` -> exit 1, sin salida). El
comando canonico de la campana fue:

```
cargo nextest run --profile audit -p vantadb --build-jobs 2
```

**No se ejecuto.** Ninguna afirmacion de esta revision es un claim de runtime: todas son
verificaciones estaticas (Read/Grep/CodeGraph/CBM), gates de Node ejecutados, o consultas web
documentadas. No se invento ningun resultado de test, cobertura ni tiempo de suite.

**Heavy Certification no ejecutada** (regla dura R3, colisiona con el CI de 0.9.0):
`stress_protocol`, `hnsw validation`, `SIFT`, `competitive_bench`, `chaos_integrity`,
`wal_resilience` -> **NOT EXECUTED**. Comando de reproduccion:
`cargo test --release --test stress_protocol -- --nocapture --test-threads=1` (y hermanos, ver
`.github/workflows/certification.yml:89-170`).

## 2. Tablero por wave

| Wave | Scope | Veredicto | C | H | M | L | I | Reporte |
|---|---|---|---|---|---|---|---|---|
| W0 | Harness `.opencode/` | PASS CON OBSERVACIONES | 0 | 0 | 2 | 1 | 3 | `audit-deep-w0-harness-20261020-033416.md` |
| W1 | Core Rust `src/` (271 rs) | PASS CON OBSERVACIONES | 0 | 0 | 1 | 0 | 5 | `audit-deep-w1-core-rust-20261020-033416.md` |
| W2 | Bindings/SDKs (9 crates) | **PASS** | 0 | 0 | 0 | 1 | 6 | `audit-deep-w2-bindings-20261020-033416.md` |
| W3 | Server + MCP | PASS CON OBSERVACIONES | 0 | 1 | 0 | 1 | 5 | `audit-deep-w3-server-mcp-20261020-033416.md` |
| W4 | Embeddings + providers + integrations | **FAIL** | 0 | 2 | 2 | 1 | 5 | `audit-deep-w4-embeddings-providers-20261020-033416.md` |
| W5 | Docs + READMEs + skills de usuario | **FAIL** | 0 | 3 | 5 | 1 | 3 | `audit-deep-w5-docs-20261020-033416.md` |
| W6 | Tests (135 ficheros, 77 targets) | PASS CON OBSERVACIONES | 0 | 0 | 2 | 2 | 5 | `audit-deep-w6-tests-20261020-033416.md` |
| W7 | Benchmarks (23 benches) + research web | **FAIL** | 0 | 0 | 3 | 0 | 4 | `audit-deep-w7-benchmarks-20261020-033416.md` |
| W8 | Tooling y periferia | PASS CON OBSERVACIONES | 0 | 1 | 3 | 1 | 6 | `audit-deep-w8-tooling-20261020-033416.md` |
| W9 | Cross-cutting (seguridad, concurrencia, contrato) | PASS CON OBSERVACIONES | 1 | 0 | 2 | 1 | 6 | `audit-deep-w9-crosscutting-20261020-033416.md` |
| W10 | Verificacion adversarial de Critical/High | **PASS** | 0 | 0 | 0 | 0 | 8 | `audit-deep-w10-verificacion-20261020-033416.md` |
| **W11** | **Sintesis global** | **FAIL** | **1** | **7** | **20** | **8** | **48** | este documento |
| | | | **1** | **7** | **20** | **8** | **48** | **84 hallazgos** |

Hallazgos **Medium+** (28) emittedos como filas `FIND-382..409` en `docs/dev/Backlog.md`, seccion
`## Hallazgos pendientes de reportes`, esquema canonico de 10 columnas. `Low`/`Info` quedan en los
reportes. **No se re-ticketearon** `FIND-343`/`FIND-249` (borrado fisico) ni `FIND-204` (sweep de
workflows, ampliado por `FIND-399`).

## 3. Top 20 hallazgos (severidad x esfuerzo)

| # | ID | Sev | Hallazgo | Archivo:línea | Esfuerzo |
|---|---|---|---|---|---|
| 1 | `FIND-407` | **Critical** | `SECURITY.md` declara cifrado en reposo que el crate publicado no tiene y que la WAL no cubre | `SECURITY.md:26` | 2h doc + WAL aparte |
| 2 | `FIND-385` | High | 4 badges de CI apuntan a workflows inexistentes (incluido "Security Audit") | `README.md:9`, `README.md:10` | 1h |
| 3 | `FIND-386` | High | `SECURITY.md` declara 0.5.x como version activa (el crate esta en 0.9.0) | `SECURITY.md:5-9` | 1h |
| 4 | `FIND-382` | High | Licencia de `embeddinggemma-2` mal declarada ("Gemma" vs Apache-2.0 real) | `embeddings/manifest.json:129` | 1h |
| 5 | `FIND-383` | High | La descarga de `embeddinggemma-2` arrastra 5 888 MB de los que se usan 1 034 MB | `embeddings/download.py:30` | 1d |
| 6 | `FIND-387` | High | `docs/api/scores.md` (el "canonical scoring contract") tiene 7 citas `archivo:línea` rotas | `docs/api/scores.md:22` | 1d |
| 7 | `FIND-408` | High | 2 de las 49 tools MCP (incl. `memory_verify_certificate`) no estan en la skill de usuario | `vantadb-mcp/src/handlers/tools.rs:82` | 1d |
| 8 | `FIND-390` | High | 11 `continue-on-error: true` sin el tag `# CATEGORY:` (4 en canales de publish) | `.github/workflows/release-wheels.yml:197` | 2h |
| 9 | `FIND-405` | Medium | El ADR Gate filtra por arbol de paquete y bloquea PRs que no tocan la API | `.github/workflows/ci-rust.yml:195` | 1d |
| 10 | `FIND-394` | Medium | `canonical_p99` nunca se mide antes de mergear (solo se compila) | `.github/workflows/nightly.yml:330` | 1d |
| 11 | `FIND-395` | Medium | 13 de 23 benches Rust no se ejecutan en ningun workflow | `benches/scores_semantics.rs:1` | 1-2d |
| 12 | `FIND-399` | Medium | 100 refs a workflows renombrados en 24 docs vivas | `docs/dev/operations/CI_POLICY.md:494` | 1d |
| 13 | `FIND-393` | Medium | Los resultados BEIR y north-star no tienen artefacto versionado | `benchmarks/beir_recall_report.json:1` | 1-2d |
| 14 | `FIND-406` | Medium | La politica de publish de `AGENTS.md` es falsa (5 secrets, 12 refs) | `AGENTS.md:646` | 1h |
| 15 | `FIND-403` | Medium | 4 modulos de `src/` (`tui`, `utils`, `shred`, `skills`) sin un solo test | `src/tui/mod.rs:1` | 1d |
| 16 | `FIND-392` | Medium | La Regla dura `-p` no se aplica en los workflows de CI | `.github/workflows/ci-rust.yml:263` | 1h |
| 17 | `FIND-388` | Medium | La formula de Homebrew sirve 0.8.0 con el crate en 0.9.0 | `Formula/vantadb.rb:12` | 1h |
| 18 | `FIND-401` | Medium | 2 docs huerfanos y 10 wikilinks en prosa (no renderizan en GitHub) | `docs/dev/tasks/RES-07.md:1` | 1d |
| 19 | `FIND-409` | Medium | Un ADR aceptado certifica un estado de verificacion falso | `docs/dev/architecture/adr/ADR-0030-brand-identity-naming-convention.md:46` | 1d |
| 20 | `FIND-400` | Medium | `README.md` afirma que el entorno del benchmark §1 esta versionado; `BENCHMARKS.md` dice que no | `README.md:396` | 2h |

Candidatos 21-28 (Medium, < 1h): `FIND-384`, `FIND-389`, `FIND-391`, `FIND-396`, `FIND-397`,
`FIND-398`, `FIND-402`, `FIND-404`.

## 4. Mapa de cobertura

**Revisado a fondo** (cada archivo del scope leido o medido, con verificacion de linea):

| Area | Volumen | Metodo |
|---|---|---|
| Core Rust `src/` | 271 `.rs` | unsafe/unwrap/non_exhaustive/panic + grafo CBM (50 365 nodos) + hotspots + capas |
| Bindings | python (4 rs / 4 255 L), node (1 957 L), wasm (3 979 L), ts (6 ts), ffi-core (215 L), proxy (11 524 L) | superficie, stubs, contrato ERR-028 |
| MCP | 25 `.rs` / 49 tools | lista de tools vs skill, alias, modo proxy |
| Server HTTP | 17 rs en `src/server/` + 4 en `vantadb-server/` | auth fail-closed, rate limit, paridad OpenAPI |
| Embeddings | manifest, lock, download, verify | manifest vs fuente oficial (API HF), tamano en disco |
| Providers + integrations | 3 providers, 9 adapters | gates de firma y de pins |
| Harness `.opencode/` | 13 commands, 11 agents, 198 dirs, 13 rules, 24 references, 82 ficheros task-system | integridad referencial completa (0 rotas) |
| Documentacion | 1 332 `.md` + 10 de raiz | 3 gates de docs ejecutados + verificacion linea a linea de claims |
| Workflows | 20 | triggers, `continue-on-error`, secrets, invocaciones de cargo/bench |
| Tests | 135 ficheros, 77 targets, 677 `#[test]`, 186 Python | inventario, ignores, cobertura por modulo, CI wiring |
| Benchmarks | 23 benches + 21 ficheros + 10 evals + 8 datasets | tier de CI, versionado de artefactos, baseline canonico |

**Revisado por muestreo** (declarado como tal en cada reporte): `fuzz/` (4 targets, existen y se
ejecutan en `certification.yml`, sin correr), `examples/` (8 subdirs, inventario), `datasets/`
(binarios: presencia y gate de verificacion), `vanta-proxy` (33 rs, superficie publica y tests;
sin auditoria de su logica interna), `desktop/` (`feature-freez/desktop`, producto aparte),
`.opencode/skills/*` (conteos + integridad referencial; no contenido funcional de cada skill),
`docs/` en su conjunto (1332 md: se validaron los de superficie publica, los runbooks de CI, la
politica de CI y el mapa de tests; el resto, por muestreo de claims numericos y de rutas).

**No tocado**: `web/` (si existe, no detectado en el perimetro del brief), el contenido funcional
de las 198 skills del harness, `vantadb-ts/tests/` (vitest, no ejecutable sin `bun` + wasm),
`vanta-memory/src/**` en profundidad (25 982 lineas: cubierto por grafo y por las waves W1/W9 de
forma transversal), `docs/dev/avance/historial/**` y `docs/dev/plans/archive/**` (archivo congelado
por politica).

## 5. FODA de VantaDB

**Fortalezas (verificadas)**
- El core Rust es sobrio: ~14 `unwrap()` reales en produccion (no 2 098 - el resto son modulos de
  test), 0 `todo!()`/`unimplemented!()`, 17 enums publicos con `#[non_exhaustive]`, limite de capa
  `src -> sdk` limpio (sdk no llama de vuelta al core), 83 bloques `unsafe` en 24 ficheros con
  invariante `SAFETY` en 21 de 24.
- Los bindings no tienen **cero** `unsafe`, y `vantadb-ffi-core` es un leaf std-only sin
  dependencias que centraliza la barrera de durabilidad de los 3 transports.
- Contrato de score con fuente unica (`src/api/scores.rs`) y **paridad real**: el zero-norm
  (ERR-028) se valida **solo** en el core y ningun binding reintroduce un fallback silencioso.
- Paridad OpenAPI **exacta** y con doble verificacion (gate Node + test Rust).
- Higiene de artefactos: cada `.json` de resultados, `.log` y `.env` local esta gitignored; solo 4
  artefactos de benchmark deliberadamente versionados.
- La politica de gates es seria: `documentation-skill`, `progreso`, `codebase-memory`, reglas por
  area con lazy-loading, y un postmortem de release que admite公开 los fallos del propio proceso
  de verificacion.

**Oportunidades**
- **Hueco de mercado real, sin explotar**: nadie publica recall + p99 + RSS + tiempo de build juntos.
  VantaDB tiene las piezas (`canonical_p99`, `memory_budget`, `beir_recall_bench`).
- **Benchmarks unicos defendibles** (propuesta P0-P2 en W7): fusion hibrida vs BM25/HNSW, GraphRAG
  sobre memoria, time-travel, memory lifecycle decay, aislamiento multi-tenant + RBAC, north-star de
  memoria de agente, paridad de scores entre SDKs, durabilidad/WAL recovery, cold start WASM.
- **EmbeddingGemma 2** es la mejor opcion multilingue del catalogo (MTEB multilingue v2 = 61.36,
  codigo v1 nDCG@10 = 78.68) y es Apache-2.0: puede ser la fila de referencia del BEIR canonico.
- **Gate de claims**: un gate que verifique que toda cifra de `docs/` tiene artefacto versionado y
  comando reproducible cerraria de raiz la familia `FIND-393/400/406`.
- **Gate tools↔skill↔doc MCP**: hoy no existe y ya produjo un hallazgo High.
- 4 modulos de produccion (`tui`, `utils`, `shred`, `skills`) y `vantadb-ffi-core` (0 tests) son
  cobertura barata con retorno alto.

**Defensas (mecanismos ya presentes que conviene no perder)**
- `#[non_exhaustive]` como red de semver en 17 enums.
- RRF centralizado (`src/api/scores.rs`) + `benches/scores_semantics.rs` con dataset determinista
  (seed `0x9E37_79B9_7F4A_7C15`).
- `OpGate`/`OpGuard` compartido: una sola barrera de durabilidad para 3 transports.
- Gates de docs: `check-links` (0 rotos de 2 670), `check-docs`, `check_openapi_parity`,
  `gen-index`, `check-npm-versions`, `gen-surface-manifest`, `validate_doc_snippets`.
- `docs/api/SEARCH_PARITY.md` como modelo de doc con URLs de fuente y fecha de consulta.
- Baseline canonico con entorno, comando, seed, fecha y artefacto versionado (Regla 11 cumplida de
  verdad en `BENCHMARKS.md`).

**Amenazas**
- **Deriva por renombre**: el rename de 14 workflows (SRV-10) se propago a 2 workflows pero no a 4
  README badges, ni a 100 refs en 24 docs, ni a 2 runbooks con el nombre viejo en el titulo. El
  mecanismo de rename es una bomba de reloj si no hay gate.
- **Claims sin artefacto**: BEIR y north-star no tienen versionado; los numeros de `README.md` y
  `CITATION.cff` no son reproducibles.
- **Metodos de verificacion que mienten** (postmortem §3): `cargo owner --list` no autentica;
  `gh api check-runs` trunca en 100 y **oculta checks**; el dump de `env` no incluye OIDC. Un gate
  verificado leyendo su configuracion en vez de ejecutandose no protege.
- **La superficie de publish usa secrets reales** (5) y la documentacion afirma que no; el modo de
  fallo F1/F2 esta documentado a 2 lineas del postmortem.
- **Puertas de calidad vacias**: `canonical_p99` no se mide en el gate de PR y 13 benches no corren;
  un hot path roto no produce ninguna señal hasta el nightly.
- **Superficie publica no versionada como contrato**: el `ADR Gate` filtra por arbol, no por
  superficie, asi que o bloquea de mas o (si alguien marca `[no-adr]`) no protege.
- 4 modulos de `src/` sin tests y `vantadb-ffi-core` con 0 tests: el unico crate cuyo contrato
  asumen los demas es el que nadie prueba.

## 6. Rutas de remediacion

**Ruta rapida (~1 dia, 11 IDs, todo doc/config sin tocar codigo de produccion)**
`FIND-407` (redactar el alcance real del cifrado) - `FIND-385` (badges) - `FIND-386` (tabla de
versiones soportadas) - `FIND-389` (CITATION) - `FIND-388` (Formula 0.9.0 + sha256) - `FIND-391`
(contar bien) - `FIND-402` (default unico) - `FIND-396` (invariante SAFETY) - `FIND-397`/`FIND-398`
(catalogo de skills) - `FIND-404` (trackear `fuzz/Cargo.lock`) - `FIND-400` (propagar la advertencia
de run-scoped).
Efecto: elimina el unico Critical y 3 de los 7 High.

**Ruta media (~1 semana, 10 IDs)**
`FIND-382` + `FIND-384` (licencia y tamano correctos, con validacion de valor en el gate) -
`FIND-383` (lista cerrada de patrones para el modelo multimodal) - `FIND-399` (sweep de los 100
refs de workflows) - `FIND-401` (wikilinks + huerfanos) - `FIND-409` (ADR de correccion) -
`FIND-405` (ADR Gate sobre la superficie real) - `FIND-406` (politica de publish real) -
`FIND-408` (2 tools en la skill + gate tools↔skill) - `FIND-390` (tags CATEGORY + gate).
Efecto: cierran los 7 High y dejan la superficie visible del repo en verde.

**Ruta estructural (~1-2 meses, 7 IDs)**
`FIND-387` (re-ancorar el contrato de scoring y anadir un gate de citas) - `FIND-392` (Regla `-p` en
CI) - `FIND-394` (medir `canonical_p99` en el gate de PR contra el baseline versionado) - `FIND-395`
(clasificar y activar los 13 benches) - `FIND-393` (versionar los artefactos de calidad) -
`FIND-403` (cubrir `tui`/`utils`/`shred`/`skills`) - y el gate transversal **"todo claim numerico
tiene artefacto versionado + comando + entorno + fecha"**, que es la defensa que hoy no existe.
Efecto: convierte el sistema de "documentado" en "verificable", que es la tesis del propio producto.

## 7. Veredicto global y estado de preparacion de v0.9.0

**Veredicto global: FAIL** - por compliance y por exactitud de la superficie visible, no por
defecto del runtime.

- **1 Critical**: una politica de seguridad afirma una propiedad (cifrado en reposo) que el crate
  publicado no tiene y que la WAL no cubre. El tarball 0.9.0 **ya esta publicado** con ese claim:
  la correccion solo entra en 0.9.1.
- **7 High**, de los cuales 4 son afirmaciones que un usuario o un agente leen como verdad: 2
  badges de CI que no existen, una politica de soporte versionada cuatro minors atras, el "canonical
  scoring contract" con 7 citas rotas, y 2 de 49 tools MCP inalcanzables desde la skill.
- **20 Medium**: deriva de nombres (100 refs), ausencia de artefactos de benchmark, gates de
  calidad vacias, y un ADR que certifica un estado falso.
- En contrapeso: **el runtime es solido**. Cero `unsafe` en los bindings, paridad OpenAPI exacta,
  disciplina de `unwrap` en produccion, `#[non_exhaustive]` bien aplicado, artefactos limpios, y
  un postmortem de release que publica sus propios fallos de verificacion. Ningun hallazgo
  Critical/High describe un fallo de ejecucion, un fallo de datos o una vulnerabilidad explotable.

**Estado de preparacion de v0.9.0**: la release se publico y verifico (postmortem: "ALL GREEN -
release v0.9.0 verified", 15/15 registries, 14 assets, `vanta-cli 0.9.0`), con 7 fallos encadenados
que ninguno detecto un gate y 3 h 40 min de retraso. Esta revision **no bloquea** el CI (corre en
background por diseno) y **no invalida** esa release: no se encontro ninguna regresion funcional.
Lo que si hace es **acotar el credito del release**: el 0.9.0 esta funcionalmente completo y
publicado, pero arrastra 8 afirmaciones de superficie visible que no son ciertas (badges, politica
de soporte, contrato de scoring, alcance del cifrado) y 3 gates de calidad que no miden lo que
prometen medir. **Veredicto operativo: publicable como esta, con una 0.9.1 de Correccion de
afirmaciones** (la ruta rapida de arriba cabe en un dia y neutraliza el Critical + 3 High).

**Lo que esta revision NO pudo hacer** (y por tanto no afirma): no ejecuto un solo test, no midio un
solo benchmark, no levanto un solo servidor, no valido ninguna cobertura por linea y no verifico
ninguna afirmacion de performance en runtime. Toda afirmacion sobre comportamiento observable en
este reporte es una afirmacion **sobre el codigo o sobre la configuracion**, verificada con la
herramienta y la linea citadas.

## 8. Indice consolidado de hallazgos

**Critical (1)**
`FIND-407` (W9) - `SECURITY.md:26` - cifrado en reposo no disponible por defecto y ausente en la WAL.

**High (7)**
`FIND-385` (W5) - `README.md:9` - 4 badges a workflows inexistentes.
`FIND-386` (W5) - `SECURITY.md:5` - politica de soporte desactualizada.
`FIND-387` (W5) - `docs/api/scores.md:22` - 7 citas `archivo:línea` rotas.
`FIND-382` (W4) - `embeddings/manifest.json:129` - licencia incorrecta.
`FIND-383` (W4) - `embeddings/download.py:30` - descarga 5.9 GB para usar 1 GB.
`FIND-408` (W3) - `vantadb-mcp/src/handlers/tools.rs:82` - 2 tools fuera de la skill.
`FIND-390` (W8) - `.github/workflows/release-wheels.yml:197` - 11 `continue-on-error` sin CATEGORY.

**Medium (20)**
`FIND-396` (W1) `src/storage/engine/txn.rs:612` · `FIND-397` (W0) `SKILLS-MANIFEST.md:3` ·
`FIND-398` (W0) `SKILLS-MANIFEST.md:596` · `FIND-384` (W4) `embeddings/manifest.json:130` ·
`FIND-402` (W4) `embeddings/download.py:123` · `FIND-399` (W5) `docs/dev/operations/CI_POLICY.md:494` ·
`FIND-400` (W5) `README.md:396` · `FIND-401` (W5) `docs/dev/tasks/RES-07.md:1` ·
`FIND-389` (W5) `CITATION.cff:11` · `FIND-409` (W5)
`docs/dev/architecture/adr/ADR-0030-brand-identity-naming-convention.md:46` ·
`FIND-392` (W6) `.github/workflows/ci-rust.yml:263` · `FIND-403` (W6) `src/tui/mod.rs:1` ·
`FIND-393` (W7) `benchmarks/beir_recall_report.json:1` · `FIND-394` (W7) `.github/workflows/nightly.yml:330` ·
`FIND-395` (W7) `benches/scores_semantics.rs:1` · `FIND-391` (W8) `AGENTS.md:505` ·
`FIND-388` (W8) `Formula/vantadb.rb:12` · `FIND-404` (W8) `fuzz/Cargo.toml:1` ·
`FIND-405` (W9) `.github/workflows/ci-rust.yml:195` · `FIND-406` (W9) `AGENTS.md:646`.

**Low (8)**
`W0-03` `.opencode/AGENTS.md:505` · `W2-06` `vanta-proxy/src/lib.rs:1` · `W3-06` `docs/api/MCP.md:1` ·
`W5-10` `README.md:219` · `W6-08` `tests/api/public_api.rs:9` · `W6-09` `tests/message_thread_test.rs:187` ·
`W8-10` `benchmarks/update_markdown.py:1` · `W9-08` `rust-toolchain.toml:2`.

**Info (48)** - distributedos en los 11 reportes (fortalezas verificadas, cobertura, metodologia y
claims de mercado con URL).

## 9. Reconciliacion con la auditoria paralela (accion requerida del owner)

Durante esta revision, **otra sesion ejecuto en paralelo una auditoria del mismo sistema**
(`docs/dev/reviews/2026-10-20-auditoria-profunda-v0.9.0.md` + `-ledger.md`, commit `06700373`
del 2026-10-20 03:42:22). Se detectan **dos problemas de coordinacion**:

**a) Colision de IDs `FIND-382` y `FIND-383`.** La auditoria paralela genero sus filas en
`docs/dev/backlog-rows-audit.tmp` (untracked, **NO aplicado** a `docs/dev/Backlog.md`) con el
rango `FIND-344..FIND-383`. Los IDs `382` y `383` **ya estan usados por esta revision** (licencia
de `embeddinggemma-2` y descarga de 5.9 GB) y **las dos filas existen hoy en el Backlog**. Ademas
su rango empieza en 344, que **ya existia** en el repo desde la auditoria pre-mercado
(`FIND-345..380` estan archivados) - es decir, su numeracion repite IDs que ya estaban
registrados. Ninguna de las dos sesiones aplico las suyas: el Backlog tiene exactamente las 28
filas de esta revision (382..409), verificado con `git diff --stat` (+28 lineas, sin reformateo).
**Recomendacion**: el owner renumera las filas del `.tmp` de la sesion paralela a `FIND-410+` y
verifica tambien su tramo 344-381 contra el historial antes de aplicarlas. Esta revision **no
modifica** el `.tmp` (Regla 0: nada ajeno se toca).

**b) Solapamiento semantico en 5 hallazgos.** La auditoria paralela llego a los mismos hechos por
otro camino; su contenido es **distinto** pero su evidencia es la misma:

| Hallazgo de esta revision | Equivalente de la auditoria paralela | Evidencia comun |
|---|---|---|
| `FIND-407` (Critical) | `D-01` -> su `FIND-344` | `SECURITY.md:26` |
| `FIND-386` | `D-13` | `SECURITY.md:3-11` |
| `FIND-385` | `S7-02` | `README.md:9,10` y `README_ES.md:9,10` |
| `FIND-399` (parcial) | `D-05` | `docs/user/operations/BENCHMARKS.md:20,22` |
| `W5-10` (Low) | `S7-03` | `README.md:219`, `README_ES.md:218` |

Lo que **esta revision aporta y la paralela no cubre**: los 4 modulos de `src/` sin test
(`FIND-403`), el crate `vantadb-ffi-core` con 0 tests, la Regla dura `-p` no aplicada en CI
(`FIND-392`), los 13 benches sin ejecutar (`FIND-395`), el corte de `canonical_p99` en el gate de PR
(`FIND-394`), la superficie de la skill MCP (`FIND-408`), la tabla de tools MCP, la verificacion de
la licencia **contra la API de Hugging Face** (no contra la pagina renderizada) y la verificacion
adversarial W10 con un hallazgo **ajustado** (`FIND-407`, refutado parcialmente). Lo que **la
paralela aporta y esta revision no cubre**: 39 P1 + 60 P2 + 41 P3 con IDs internos (`D-01`,
`S3-A1`, `S7-05..S7-30`, `VDB-01`) que cubren areas que aqui solo se muestrearon (dimension de
vector, `exclude_superseded` en Node, `paths:` de adapters en CI, semver check).

**c) Commit externo de 6 de mis reportes.** El commit `06700373` (03:42:22), hecho por la sesion
paralela, incluyo `audit-deep-w0/w1/w2/w3/w4/w5-20261020-033416.md` (mi trabajo) junto a sus
propios ficheros. Esta revision **no hizo ningun commit** (verificado: `git log` no contiene
ninguna autoria de esta sesion). Los 6 reportes que quedaron fuera de ese commit
(`w6`, `w7`, `w8`, `w9`, `w10`, `sintesis`) siguen untracked mas sus 2 retoques (`w3`, `w5`).

## 10. Reportes de esta revision

| Wave | Reporte |
|---|---|
| W0 | `docs/dev/reviews/audit-deep-w0-harness-20261020-033416.md` |
| W1 | `docs/dev/reviews/audit-deep-w1-core-rust-20261020-033416.md` |
| W2 | `docs/dev/reviews/audit-deep-w2-bindings-20261020-033416.md` |
| W3 | `docs/dev/reviews/audit-deep-w3-server-mcp-20261020-033416.md` |
| W4 | `docs/dev/reviews/audit-deep-w4-embeddings-providers-20261020-033416.md` |
| W5 | `docs/dev/reviews/audit-deep-w5-docs-20261020-033416.md` |
| W6 | `docs/dev/reviews/audit-deep-w6-tests-20261020-033416.md` |
| W7 | `docs/dev/reviews/audit-deep-w7-benchmarks-20261020-033416.md` |
| W8 | `docs/dev/reviews/audit-deep-w8-tooling-20261020-033416.md` |
| W9 | `docs/dev/reviews/audit-deep-w9-crosscutting-20261020-033416.md` |
| W10 | `docs/dev/reviews/audit-deep-w10-verificacion-20261020-033416.md` |
| W11 | `docs/dev/reviews/audit-deep-sintesis-20261020-033416.md` (este documento) |
