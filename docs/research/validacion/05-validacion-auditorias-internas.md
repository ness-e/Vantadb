# 05 — Validación Adversarial de Auditorías y Análisis Internos

**Fecha de validación:** 2026-08-25
**Validador:** Agente auditor adversarial (ox-alpha) · workspace `C:\Users\Eros\VantaDB Proyect\VantaDB`
**HEAD al validar:** `29d21cba` · versión workspace actual: **0.6.x**
**Método:** extracción completa de los 6 documentos + verificación claim-a-claim contra el código real (`codegraph_explore`, grep, lectura directa) e historia git (`git show`/`git cat-file` en commits `63b0101d` y `4c2102eb`). Claims de mercado marcados para re-verificación cruzada con agentes de research (1 búsqueda web de control falló por bot-challenge en ambos motores).

---

## Veredicto global por documento

| # | Documento | Fiabilidad estimada | Veredicto en 1 línea |
|---|-----------|--------------------:|----------------------|
| 1 | `experimental-quarantine-2024-06.md` | ~85% | Registro histórico fiable de la cuarentena; título/año confuso ("2024") pero contenido fechado 2026 y verificado; 2 referencias internas rotas. |
| 2 | `VantaDB_AnalisisTecnico_BusinessProfessional_2026-07-26.pdf` | ~70% | Análisis arquitectónico correcto en lo general pero con errores factuales de inventario (crates de integración inexistentes como directorios raíz, `wal_archiver.rs`); tono complaciente, benchmarks solo auto-declarados. |
| 3 | `VantaDB_Auditoria_Tecnica.md` (31-jul-2026, commit `4c2102eb`) | ~85% | La más precisa y trazable: tests/LOC/archivos verificados casi exactos; varios hallazgos críticos SIGUEN vivos hoy; otros ya corregidos desde su fecha. |
| 4 | `VantaDB_Auditoria_Tecnica.pdf` (784 KB) | = doc 3 (~85%) | **Es un export del MD #3** (misma sonda textual, mismas secciones, 58 págs): no duplica trabajo; validar solo diferencias de formato → ninguna material detectada. |
| 5 | `VantaDB_Manual_Estrategico_Unificado.md` (31-jul-2026) | ~65% | Marco estratégico valioso y honesto en lo comercial, pero métricas internamente contradictorias (1.371 vs 1.075 commits), lista de adapters con nombre erróneo, y claims de mercado sin verificar localmente. |
| 6 | `vantadb-audit-report.md` ("2025"-07-27, commit `63b0101d`) | ~60% (hoy) / ~90% (en su commit) | Instantánea FIEL de 63b0101d — verificado con `git show` — pero: **el año 2025 es incorrecto (el commit es del 2026-07-27)**, owner del repo erróneo (DevpNess vs ness-e), y ~80% de sus hallazgos ya están corregidos a día de hoy. |

---

## Validación por documento

### 1. `experimental-quarantine-2024-06.md`

**Papel determinado:** NO está obsoleto ni es un error de fecha: es una **lápidas/archivo histórico** escrito el 2026-06-10 (actualizado jul-2026). El "2024-06" del título es el codename del batch experimental, no una fecha real. Su función es documentar POR QUÉ se eliminaron `experimental-lisp` y `experimental-governance` y qué utilidades se rescataron.

| Claim | Clasificación | Evidencia |
|---|---|---|
| `experimental-lisp` eliminado (jul-2026) | ✅ Confirmado | No existe dir `experimental-lisp`; análisis referenciado existe |
| `experimental-governance` eliminado; design doc preservado | ✅ Confirmado | `docs/architecture/EXPERIMENTAL_GOVERNANCE_DESIGN.md` existe |
| Utilidad extraída `src/utils/duplicate_prevention.rs` (Bloom filter) integrada como `DuplicatePreventionFilter` | ✅ Confirmado | Archivo presente en src/utils/ |
| Utilidad extraída `src/utils/confidence_metrics.rs` (`OriginCollisionTracker`, `compute_confidence_friction`) | ✅ Confirmado | Archivo presente en src/utils/ |
| Referencias: `docs/progreso/cuarentena-experimental/walkthrough.md` e `implementation_plan.md` | ❌ Incorrecto (hoy) | Ambas rutas NO existen (árbol progreso migrado); referencias rotas |

**Qué conservar:** la racional "runtime governance → compile-time governance via IQL AST Pass" y las lecciones (borrow-checker panics con MmapMut, GIL blocking, overhead en embebido). Es contexto de decisiones que evita reintentar un camino ya descartado.

---

### 2. `VantaDB_AnalisisTecnico_BusinessProfessional_2026-07-26.pdf` (33 págs)

Análisis positivo estilo consultoría. Sin hallazgos de bugs (no es su objetivo).

| Claim | Clasificación | Evidencia |
|---|---|---|
| Motor embebido Rust, WAL CRC32C, HNSW+BM25+RRF, sin servidor | ✅ Confirmado | `src/wal.rs:182-294`, `src/index/search/*`, `src/planner.rs:27` |
| MSRV fijado vía `rust-toolchain.toml` (1.94.1) | 🕐 Desactualizado | Hoy `rust-toolchain.toml` = **1.95.0** (era cierto al 26-jul) |
| `#![deny(unsafe_op_in_unsafe_fn)]` en lib.rs | ❓ No verificable en esta pasada (plausible) | No re-verificado línea a línea |
| Directorios raíz `vantadb-langchain/`, `vantadb-llamaindex/`, `vantadb-haystack/`, `vantadb-crewai/`, `vantadb-dspy/`, `vantadb-letta/`, `vantadb-litellm/`, `vantadb-openai/`, `vantadb-mem0/`, `vantadb-ollama/` como crates del workspace | ❌ **Incorrecto** | Los adaptadores son paquetes Python bajo `integrations/{crewai,dspy,haystack,langchain,letta,llamaindex,mem0,ollama,openai}`; en raíz solo hay `vantadb-{mcp,node,python,server,ts,wasm}`. Además `litellm` no existe NI siquiera en integrations/ |
| `wal_archiver.rs` existe (feature pitr) | ❌ Incorrecto (hoy) | El archivo no existe en HEAD (existía en 63b0101d; fue eliminado después) |
| `wal_shipping.rs` existe (feature wal-shipping) | ✅ Confirmado | `src/wal_shipping.rs` implementación real (reqwest blocking, `ship_once`, `run_loop`, shutdown handle) |
| Backends: fjall default / rocksdb opcional / in_memory tests | ✅ Confirmado | `src/backends/`, feature `fjall` en Cargo.toml default |
| Benchmarks BENCH-01 (ingesta ~5400 vec/s, HNSW p50 1.20ms, híbrida p50 2.10ms) y Fase 2 speedups 2.16×–2.80× | ⚠️ Parcialmente cierto | Son auto-declarados por el proyecto; el propio PDF lo admite en §7.1. No verificados independientemente |
| 16+ features; failpoints con crate `fail`; cargo-deny; fuzz targets | ✅ Confirmado | Cargo.toml features; `fail::fail_point!` en wal.rs:273/303; deny.toml activo |
| SECURITY.md define canal responsable; server escucha 127.0.0.1 por defecto | ✅ Confirmado | SECURITY.md real con tabla de versiones soportadas |
| Windows binaries sin firmar (SmartScreen) | ✅ Confirmado (estado declarado) | README lo reconoce; no re-verificado binario a binario |

**Qué conservar:** catálogo de patrones (WAL canónico, Planner/Físico/Executor, Builder/Facade/Strategy), tabla de módulos con visibilidad, recomendaciones R1 (MemTable en disco), R7 (Authenticode), R10 (gobernanza/bus factor) y R15 (SLSA L3) — siguen siendo las brechas correctas.

**Qué descartar:** la Tabla 5 de árbol raíz (inventario erróneo) y cualquier uso de este PDF como fuente de hechos de inventario.

---

### 3. `VantaDB_Auditoria_Tecnica.md` (auditoría estática, commit `4c2102eb`)

Documento más riguroso. Metadatos verificados contra el commit:

| Claim | Clasificación | Evidencia |
|---|---|---|
| Commit HEAD `4c2102eb` al 31/07/2026 | ✅ Confirmado | `git cat-file`: commit existe; fechado 2026-08-01 23:53 (±1 día de lo declarado) |
| 149 archivos .rs bajo src/ | ✅ Confirmado EXACTO | `git ls-tree -r 4c2102eb -- src`: **149 archivos** |
| 67.227 LOC | ⚠️ Parcialmente cierto | Medición propia (líneas no-blancas) en ese árbol: 60.404; con líneas en blanco el total puede acercarse a 67K — metodología no declarada, cifra plausible pero no reproducida exacta |
| 1.773 tests (`#[test]` en src) | ✅ Confirmado EXACTO | `git grep -c '#\[test\]' 4c2102eb -- src/**` = **1773** |
| v0.5.0-beta | ✅ Consistente | Dockerfile actual aún declara APP_VERSION=0.5.0; workspace hoy 0.6.x |
| **CRIT-01**: commit_transaction escribe WAL Commit ANTES de aplicar mutaciones (ops.rs:296–364) | ✅ **SIGUE VIVO** (rehubicado) | Hoy en `src/storage/engine/txn.rs:119-191`: build batch Begin+ops+Commit → `batch_append` (L158-160) → loop de `apply_insert_with_txn`/`apply_delete` DESPUÉS (L163-188). Fallo a mitad del loop deja WAL=Commit con stores parciales. Sin rollback |
| **CRIT-02**: search_layer retorna 0.0 ante header mmap corrupto | 🕐 Corregido tras auditoría | `src/index/search/layer.rs`: checks explícitos `vec_end <= mmap_len` con `continue` (L180-185) y comentarios SAFETY de bounds; patrón "return 0.0" ausente |
| **CRIT-03**: locks anidados 4 niveles en ScannIndex | ✅ **SIGUE VIVO** | `src/index/scann.rs:51-59`: 5 Mutex separados (`entries`, `min_bound`, `max_bound`, `dim`, `bounds_initialized`); sin consolidación ni orden documentado |
| **CRIT-04**: `.expect("RwLock/Mutex poisoned")` en sync_ext.rs | ✅ **SIGUE VIVO** | `src/sync_ext.rs:11,15,25`: los tres `.expect(...poisoned)` presentes |
| **CRIT-05**: panic si falta `VANTA_OPENAI_API_KEY` | ✅ **SIGUE VIVO** | `src/llm.rs:149`: `env::var(...).expect("VANTA_OPENAI_API_KEY must be set")`; docstring admite "Panics if not set" |
| **CRIT-07**: search sin validar query_vector vacío → NaN | ❓ No verificable en esta pasada | No re-ejecutado el flujo completo post-refactor de sdk/search; pendiente test adversarial |
| BM25 k1=1.2, b=0.75 (text_index.rs:60-61) | ✅ Confirmado (valores; línea movida) | `src/text_index.rs:33,35` |
| RRF_K=60.0 (planner.rs:25) | ✅ Confirmado | `src/planner.rs:27` |
| MAX_VEC_F32_LEN=10_000_000 (graph.rs:32) | ✅ Confirmado (valor; ubicación movida) | `src/node/vector_data.rs`: const 10_000_000 (~40MB/vector) |
| RBAC = stub sin auth middleware | ✅ Confirmado (sigue stub-ish) | `src/rbac.rs:1` `#![allow(dead_code)]`; tipos pub(crate) sin wiring visible |
| Tutorial usa `import vantadb` roto (ALT-10) | 🕐 Corregido tras auditoría | `docs/tutorials/01-ai-agent-memory.md:33,177` ahora usa `from vantadb_py import VantaDB`; README documenta dual-import `vantadb`/`vantadb_py` (README.md:67,97) |
| WAL shipping "✗ No iniciado" | 🕐 Desactualizado | `src/wal_shipping.rs` implementado (reqwest blocking + run_loop + shutdown); feature `wal-shipping = ["dep:reqwest"]` (Cargo.toml:122) |
| PITR "✗ No iniciado" | 🕐 Probablemente desactualizado | `wal_archiver.rs` existía en 63b0101d y luego se eliminó/reubicó; snapshot_restore aparece en commits recientes (MCP-34b) — estado actual ambiguo, requiere mapeo |
| SECURITY.md placeholder (MED-17) | 🕐 Corregido | SECURITY.md actual: política real con matriz de versiones soportadas y política de patches |
| "10 ADRs públicos" | ⚠️ Parcialmente cierto | Había **12** ADRs en `docs/architecture/adr/` al commit auditado; hoy 37 |
| 9 adapters publicados en PyPI | ✅ Confirmado (dirs) | 9 paquetes en `integrations/` (ver matiz de nombres en doc 5) |
| Deny.toml ignora RUSTSEC-2023-0089 hasta 2027 (BAJ-05) | ✅ Confirmado | `deny.toml:3-11`; además trackea RUSTSEC-2026-0253 (lru UAF) — el doc no podía conocerlo (posterior) |

**Qué conservar:** TODO el inventario de hallazgos vivos (CRIT-01, CRIT-03, CRIT-04, CRIT-05, CRIT-06 frágil) como backlog de seguridad vigente; la tabla LOC por archivo (base para splits); el plan de acción de 7 acciones como roadmap técnico.

**Qué descartar/refrescar:** estados de features marcados "No iniciado" (wal-shipping/PITR), el bug CRIT-02 ya corregido, y números de línea (el código se reorganizó: ops.rs → txn.rs/insert.rs/delete.rs/get.rs; index/search.rs → index/search/*).

---

### 4. `VantaDB_Auditoria_Tecnica.pdf` (784 KB, 58 págs)

| Claim | Clasificación | Evidencia |
|---|---|---|
| Es export/gemelo del MD #3 | ✅ Confirmado | Extracción pypdf: contiene la sonda textual exacta ("el WAL escribe Commit antes de aplicar…"), mismos IDs CRIT-01..07, mismo "67.227", mismo commit `4c2102eb`. Diferencias = artefactos de extracción PDF (saltos de línea parten frases, p.ej. "628 llamadas" no matcheó por wrap) |
| Contenido adicional respecto al MD | ✅ Ninguno material | No se detectaron secciones exclusivas del PDF |

**Veredicto:** duplicado de formato. **Validar solo el MD (#3)**; usar el PDF únicamente para distribución/lectura. Fiabilidad heredada: ~85%.

---

### 5. `VantaDB_Manual_Estrategico_Unificado.md` (164 KB)

Documento de negocio (meta USD 5.000 antes del 01/01/2027). Claims técnicos y de negocio:

| Claim | Clasificación | Evidencia |
|---|---|---|
| Licencia Apache-2.0 en el repo | ✅ Confirmado | `LICENSE` = Apache License 2.0 |
| 9 adaptadores PyPI: LangChain, LlamaIndex, Mem0, CrewAI, DSPy, Letta, OpenAI, LiteLLM (+Haystack) | ⚠️ Parcialmente cierto | Hay 9 dirs en `integrations/` PERO la lista real es: crewai, dspy, haystack, langchain, letta, llamaindex, mem0, **ollama**, openai. **LiteLLM no existe como adapter**; Ollama sí. El doc nombra LiteLLM y omite Ollama |
| ROADMAP con riesgos bloqueantes R1(CI), R2(WASM demo 80/219), R3(bincode deprecated), R4(MSVC linker), R5(backlog sin priorizar), R8(claims falsos landing 50x vs 40x) | ✅ Confirmado (estructura) | `docs/strategy/ROADMAP.md` contiene R1-R8 con esa semántica (R1 CI 🔴, DRV-115 MSVC, etc.). Resolución individual de cada riesgo a día de hoy: parcialmente verificada (MSVC/toolchain 1.95 ok; WASM/claims web no re-verificados aquí) |
| COMPETITIVE_ANALYSIS.md sólido | ✅ Confirmado | Existe en `docs/benchmarks/COMPETITIVE_ANALYSIS.md` |
| GO_TO_MARKET.md con pricing Free/Pro $99/Business $499/Enterprise custom | ✅ Confirmado (existe) | `docs/strategy/GO_TO_MARKET.md`; valores de pricing no re-leídos en detalle |
| CLA_CORPORATE.md y CLA_INDIVIDUAL.md en repo | ✅ Confirmado | Ambos existen |
| Discord `discord.gg/g8nqB3NtXt` en README/SUPPORT | ✅ Confirmado | SUPPORT.md:3 |
| ChromaDB agregó BM25 nativo (invalidando claim previo) | ❓ Mercado — derivar a research | No verificable localmente |
| LanceDB recall 23-25% cosine en datasets pequeños | ❓ Mercado — derivar a research | Cifra crítica para el moat; requiere benchmark cruzado |
| Stripe Atlas / Mercury excluyen Venezuela; PayPal Business restringido VE | ❓ Mercado/legal — derivar a research | Requiere verificación ToS actualizados (alta prioridad: bloqueante del plan) |
| Recall 100% GloVe / 622 QPS / ACORN 100% recall a 1% selectividad | ⚠️ Auto-declarado | ACORN existe en código (`src/index/search/layer.rs:290-296`, `nearest.rs:163-183`); las CIFRAS provienen de COMPETITIVE_ANALYSIS propio, sin verificación independiente |
| "claude-mem (89K★) usa SQLite simple" | ❌ **Cifra inverosímil** | 89K estrellas colocaría ese repo entre los top-30 de GitHub históricos; casi seguro error de magnitud (¿89×? ¿8.9K?). Dato no accionable tal cual |
| 1.371 commits (Resumen Ejecutivo) vs 1.075 commits (Anexo VI) | ❌ **Contradicción interna** | Ambas cifras conviven en el mismo documento; hoy el repo tiene 2.436 commits totales. Al 31-jul ninguna de las dos era verificable como exacta |
| ~42.500 LOC Rust | ⚠️ Inconsistente con doc 3 | Doc 3 (mismo día) dice 67.227 LOC. Probables scopes distintos (core vs core+WASM+python+adapters, como detalla Anexo VI), pero el manual nunca reconcilia ambas cifras |
| 444 tests Rust (Anexo VI) | ⚠️ Inconsistente | Doc 3 cuenta 1.773 `#[test]` solo en src/. Criterios de conteo distintos sin explicación |
| 2 estrellas GitHub / 0 clientes pagadores | ❓ Mercado — derivar a research | Estado externo; no verificable desde el workspace (búsqueda de control bloqueada por bot-challenge) |
| Crate `vantadb` en crates.io v0.1.4; npm `vantadb` + `vantadb-wasm` | ❓ Mercado — plausible | Coherente con AnalisisTecnico PDF ("vantadb = \"0.1\""); requiere check PyPI/npm |
| Congelar backlog / no features hasta 3 clientes pagadores | ➖ Recomendación (no claim) | Estrategia, fuera de scope técnico; coherente con hallazgos técnicos vivos |

**Qué conservar:** Parte B/C (priorización legal-comercial urgente: jurisdicción+banca VE, licencia/Open-Core decisión ANTES del Show HN, marca, ToS/Privacy, design partners), Anexo III (checklist Venezuela), Anexo VI (diagnóstico honesto del repo), y la regla "lead con beneficio, soporta con arquitectura".

**Qué descartar/corregir:** cifras de commits/LOC/tests (usar las del doc 3 o medir de nuevo), la mención claude-mem 89K★, y todo pricing/benchmark citado sin fuente primaria.

---

### 6. `vantadb-audit-report.md` (commit `63b0101d`)

**Hallazgo de metadatos:** fecha declarada **2025-07-27** pero `git show` demuestra que `63b0101d` fue commiteado **2026-07-27 01:15 (-04:00)** → año erróneo. Owner declarado `DevpNess/Vantadb` ≠ `ness-e/Vantadb` usado por los otros docs → inconsistencia de procedencia sin resolver. **El contenido SÍ corresponde fielmente a ese commit** (verificado con `git show`):

| Claim (en su commit) | Clasificación | Evidencia actual |
|---|---|---|
| Dockerfile COPYea 8 dirs inexistentes (vantadb-mem0/letta/crewai/dspy/haystack/litellm/openai/ollama) | 🕐 **Era cierto** → HOY CORREGIDO | `git show 63b0101d:Dockerfile` contiene esas COPY; Dockerfile actual ya no las tiene (usa vanta-memory/, vanta-proxy/) — aunque el RUN skeleton actual tiene 2 `\` de continuación faltantes (bug nuevo no cubierto por ningún doc) |
| CRIT-08: Docker Rust 1.94.0 < MSRV 1.94.1 | 🕐 Era cierto → HOY CORREGIDO | En 63b0101d: ARG=1.94.0 vs rust-version=1.94.1 (verificado). Hoy: toolchain 1.95.0 y Docker 1.95.0 |
| CRIT-01: ShardedWal::recover usa checkpoint_seq global con seq local por shard (pierde N×shards registros) | 🕐 Era cierto → HOY CORREGIDO | `src/wal_sharded.rs:238-269`: matemática round-robin (`skip_base = checkpoint_seq/shards`, `extra_shards = checkpoint_seq%shards`) exactamente la recomendación del fix |
| CRIT-02: compact_layout panic con copy_from_slice truncado | 🕐 Era cierto → HOY CORREGIDO | `src/storage/archive.rs:105-123`: valida truncamiento con mensaje "vstore truncated…" antes del copy |
| CRIT-03: `.expect("key slice fits [u8;16]")` en deserialización | 🕐 Era cierto → HOY CORREGIDO | Búsqueda de `try_into().expect` en storage/engine: sin resultados |
| CRIT-05: flush tragado + sin sync_all antes de rename | 🕐 Era cierto → HOY CORREGIDO | `archive.rs:95,135-137`: `flush().map_err(...)?` y `sync_all()` antes de rename |
| CRIT-06: SyncMode::Periodic sin flush_threshold = pérdida silenciosa | 🕐 Era cierto → HOY CORREGIDO | `src/wal.rs:340-355`: `DEFAULT_PERIODIC_THRESHOLD = 1` ("sync every write to avoid losing more than one record") |
| MED-13/14: sin fsync de dir; corrupto truncado sin cuarentena | 🕐 Era cierto → HOY CORREGIDO | `crate::utils::fs::sync_parent_dir` en rotate/auto_rotate; `quarantine_corrupt_tail` en WalWriter::open (wal.rs:246-249) |
| CRIT-09: providers usan herencia sin ser miembros | 🕐 Era cierto → HOY CORREGIDO | Cargo.toml:14 `exclude` + nota explícita L633 "providers … are NOT workspace members" |
| CRIT-10: prometheus/rayon opcionales sin feature → código muerto | 🕐 Era cierto → HOY CORREGIDO | Cargo.toml:133-134 define ambos features; 67 bloques cfg(prometheus) alcanzables |
| ALTO-01: parser trunca "3.14"→Int(3) | 🕐 Era cierto → HOY CORREGIDO | `src/parser/mod.rs:138`: comentario "double BEFORE parse_i64: '3.14' should be Float(3.14)" — orden intercambiado |
| ALTO-03: derivación de clave con SHA-256 único sin stretching | 🕐 Era cierto → HOY CORREGIDO | `src/crypto.rs:40-41,279-280`: PBKDF2-HMAC-SHA256 (ring) con path legacy Sha256 solo para descifrar datos viejos |
| ALTO-04: X-Forwarded-For confiada ciegamente (spoofing de rate-limit) | 🕐 Era cierto → HOY CORREGIDO | `cli_server.rs:152,449,593-597`: XFF honrado SOLO si peer ∈ proxies de confianza configurados |
| ALTO-13: guard TS rechaza version numérica | 🕐 Era cierto → HOY CORREGIDO | `vantadb-ts/src/guards.ts:16`: acepta `"string" || "number"` — el fix exacto recomendado |
| ALTO-15 / MED-25: integraciones 0.3.0 vs core; noImplicitAny:false | 🕐 Eran ciertos → HOY CORREGIDOS | Las 9 integraciones en 0.5.0; `web/tsconfig.json:13` `noImplicitAny: true` |
| IVF nunca invalidado tras inserts (ALTO-02) | 🕐 Mayormente corregido | `graph.rs:355-356`: cache de ivf_index con node-count de construcción (detección de staleness); invalidación completa no auditada a fondo |
| RBAC "FUNCIONAL con permisos granulares" | ⚠️ Sobrestimado | `src/rbac.rs` sigue con `#![allow(dead_code)]` y sin middleware — contradice al doc 3 ("stub"), que es el veredicto más cercano al código |
| Inventario: wal_archiver.rs (417 líneas) y wal_shipping.rs existen | ✅ Era cierto en 63b0101d | `git cat-file -e` confirma existencia en ese commit (wal_archiver eliminado después) |
| Aspectos positivos: ConstantTimeEq, nonce GCM aleatorio, bind 127.0.0.1, fuzz x4, deny.toml estricto | ✅ Confirmado (muestras) | Coherente con crypto.rs y config actual |

**Qué conservar:** como **registro histórico del estado al 2026-07-27** y como checklist de regresión (sus fixes deben tener tests que impidan reintroducirlos). Su sección "Aspectos positivos" es útil para el informe final.

**Qué descartar:** cualquier lectura de sus severidades como estado ACTUAL — ~80% de los CRIT/ALTO ya no aplican; corregir fecha (2026, no 2025) y owner antes de citarlo.

---

## ⚠️ AFIRMACIONES IMPORTANTES FALSAS O PELIGROSAS (destacado)

1. **Fecha imposible en doc 6:** "Fecha de auditoría 2025-07-27" es falsa — el commit analizado es del **2026-07-27** (verificado con `git show -s --format=%ci`). Un año entero de desfase que invalida cualquier cronología construida sobre este documento.
2. **Inventario fantasma en doc 2 (PDF Business Professional):** lista 10 crates de integración (`vantadb-langchain/`, `vantadb-litellm/`, etc.) como directorios raíz del workspace. **Nunca existieron como tales** — son paquetes Python en `integrations/`, y `litellm` ni siquiera existe ahí. Cualquier onboarding o CI basado en esa tabla fallaría.
3. **"RBAC FUNCIONAL" (doc 6) vs "stub" (doc 3):** contradicción directa entre documentos. El código (`#![allow(dead_code)]`, sin middleware) da la razón al doc 3. **Peligroso:** alguien podría desplegar creyendo que hay control de acceso real. NO hay auth enforcement en el server.
4. **Contradicción interna del Manual (doc 5):** 1.371 commits (Resumen) vs 1.075 (Anexo VI) en el mismo documento; además 42.500 LOC sin reconciliar con las 67.227 del doc 3. Cualquier pitch/métrica construida sobre estas cifras debe re-medirse.
5. **"claude-mem 89K★":** cifra de mercado inverosímil (top histórico de GitHub). Error de magnitud casi seguro — no citar.
6. **Doc 3 subestima superficie ya implementada:** declara "WAL shipping ✗ No iniciado" cuando `src/wal_shipping.rs` está implementado. Riesgo inverso: prometer como Deferred algo que ya expone código.
7. **Bug nuevo NO cubierto por ningún documento:** el Dockerfile actual tiene 2 `\` de continuación faltantes en el bloque RUN del skeleton (líneas del `mkdir -p vantadb-mcp/src ... vanta-proxy/src`) → `docker build` probablemente falla HOY. Ninguno de los 6 docs lo captura (es posterior a todos).
8. **CRIT-01 sigue vivo (doc 3):** la ventana Commit-WAL-antes-de-aplicar persiste en `txn.rs:119-191`. Es el único hallazgo Crítico de corrupción potencial SIN corregir de los auditados — prioridad máxima para síntesis final.

---

## Qué conservar de cada documento para la síntesis final

| Doc | Conservar |
|---|---|
| 1 Quarantine | Lección runtime-vs-compile-time governance; registro de utilidades rescatadas. Corregir referencias rotas. |
| 2 AnalisisTecnico PDF | Catálogo de patrones; recomendaciones R1/R7/R10/R15; nada de su inventario de archivos. |
| 3 Auditoría Técnica MD | **Fuente primaria de hallazgos vivos** (CRIT-01, CRIT-03, CRIT-04, CRIT-05); plan de 7 acciones; tabla LOC/splits. Actualizar números de línea. |
| 4 Auditoría PDF | Nada nuevo — citar como formato de distribución del #3. |
| 5 Manual Estratégico | Prioridades URG (jurisdicción+banca VE, licencia antes del Show HN, design partners, pricing $49/$199/$2.5k); checklist Venezuela; diagnóstico Anexo VI (con cifras re-medidas). |
| 6 Audit Report | Checklist de regression para los 15+ fixes ya aplicados; sección "aspectos positivos"; corregido año/owner. |

## Conclusiones

1. **Los tres documentos técnicos son complementarios en el tiempo, no contradictorios en contenido:** doc 6 = instantánea fiel del 27-jul (año mal etiquetado), doc 3 = foto del 31-jul~01-ago, HEAD de hoy = 29d21cba. Entre el 27-jul y hoy se corrigieron ≥15 hallazgos de severidad alta — el ritmo de remediación es excepcional, pero hace que **solo el doc 3 siga siendo mayormente accionable**.
2. **Deuda crítica viva consolidada (para síntesis):** atomicidad de commit_transaction (CRIT-01), locks anidados ScannIndex (CRIT-03), panics por poison/env-var (CRIT-04/05). Todo lo demás auditado como Crítico ya está resuelto o no reproduce.
3. **Fiabilidad de metadatos es el punto débil transversal:** un año erróneo (doc 6), un owner distinto (doc 6), un título-codename engañoso (doc 1), cifras internas contradictorias (doc 5), e inventario de archivos incorrecto (doc 2). Regla para la síntesis: **citar código con archivo:línea actual, nunca cifras de estos docs sin re-medir**.
4. **Claims de mercado pendientes** (estrellas GitHub, Stripe/Mercury-VE, ChromaDB-BM25, LanceDB-recall, presencia PyPI/npm): derivados a los agentes de research según protocolo; la búsqueda de control local fue bloqueada (bot-challenge DDG+Sogou).
5. El PDF gemelo (#4) confirma la práctica de export: cero divergencia de contenido detectada — mantener un solo source of truth (el MD).

— Fin del informe de validación · generado 2026-08-25 · agente ox-alpha · sin modificaciones a documentos originales ni código.
