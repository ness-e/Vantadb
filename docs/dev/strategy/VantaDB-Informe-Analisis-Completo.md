# 🔬 INFORME DE ANÁLISIS INTEGRAL — VantaDB

> **Repositorio:** https://github.com/ness-e/Vantadb · **Rama:** `develop` · **Versión:** 0.6.1
> **Método:** clonación completa + 12 agentes de análisis especializados en paralelo + verificación manual de cada hallazgo crítico + compilación local
> **Alcance:** 18 módulos, documentación (1,712 archivos .md), CI/CD (26 workflows), benchmarks, seguridad, arquitectura y análisis de producto
> **Verificación de compilación:** `cargo check -p vantadb --all-targets` → ✅ **PASS, 0 errores, 0 warnings** (7m41s, Rust 1.95.0)

---

## 1. RESUMEN EJECUTIVO

VantaDB es un motor de memoria/vector embebido en Rust para agentes de IA: **214,646 líneas de Rust**, 32,530 de TS/TSX, 18,273 de Python, 1,712 archivos de documentación (237K LOC — más documentación que código), 26 workflows de CI, 87 tools MCP, bindings para Python/Node/TS/WASM, un proxy LLM, un server HTTP y una app desktop Tauri.

**Veredicto global: 7.3/10** — un proyecto *notablemente* más disciplinado que el promedio de open source, con ingeniería de élite en el core y una cultura de honestidad excepcional (Regla 11: "claim sin fuente reproducible no existe"), pero con grietas serias de seguridad puntual, gaps de wiring entre módulos (código excelente que nadie llama), versiones stale cruzadas y un embudo de lanzamiento público literalmente roto.

| # | Módulo | LOC aprox. | Nota | Resumen en una línea |
|---|---|---:|:---:|---|
| 1 | **src/ (core engine)** | 98,842 | **8.5** | Disciplina extrema: 7 unwraps en 99K LOC prod, WAL CRC32C+auto-heal, chaos testing, Miri |
| 2 | **src/ restante** (tui, graphrag, entity…) | 19,458 | **8.0** | 98.5% de código vivo; muertos identificados: `eviction.rs` y bloom filter |
| 3 | **vanta-memory** | 30,958 | **7.5** | Pipeline L0→L3 de manual; pero ~40% (scheduler) sin host que lo ejecute |
| 4 | **vanta-proxy** | 14,160 | **7.5** | Pipeline de 16 etapas impecable; loop de memoria inerte y cost tracking fantasma |
| 5 | **vantadb-server** | ~1,930+5,656 | **8.0** | Refuse-to-start, RBAC 3 capas, JWT fail-closed; RBAC rígido, dashboard fantasma |
| 6 | **vantadb-mcp** | 20,512 | **8.0** | 87 tools verificadas, errores accionables de verdad; default `full` = 23K tokens overhead |
| 7 | **skills/** + manifest | ~1,500+602 | **6.5** | Política de recall referente; hooks multi-cliente **rotos en runtime** y fallan en silencio |
| 8 | **vantadb-node** | 3,586 | **8.0** | Napi de manual (OpGate, spawn_blocking); versión 0.5.0 stale, features asimétricas |
| 9 | **vantadb-python** | 8,087 | **8.5** | El mejor SDK: 52 `py.detach`, numpy, errores tipados con `.code/.retriable/.hint` |
| 10 | **vantadb-ts** | 6,810 | **7.5** | Validación de frontera ejemplar; `score`→`distance` documentado al revés |
| 11 | **vantadb-wasm** | ~2,500 | **8.0** | Persistencia snapshot honesta (CRC+atomic rename, Web Locks); doc crash-model stale |
| 12 | **providers/** (openai/ollama/litellm) | 2,274 | **7.5** | Ingeniería fina; no publicados en PyPI, ~60% copy-paste, colisión de nombres |
| 13 | **integrations/** (9 adapters) | ~2,500 | **7.0** | Adapters reales con tests; nunca publicados (`adapters-v*` no existe), import deprecado |
| 14 | **examples/** | ~3,000 | **6.0** | 2 demos sobresalientes; Colab notebook **roto** (API removida), 6 esqueletos duplicados |
| 15 | **embeddings/** | ~600 | **8.0** | 9 modelos rev-pinned con lock file; mecánica de verificación en 2 niveles |
| 16 | **benchmarks/ (Python)** | ~800 | **3.5** | **Script roto desde el rename 0.5.0** — causa raíz de la brecha 30× en números públicos |
| 17 | **benches/ (criterion)** | 4,242 | **8.5** | Metodología verificada al detalle; 13/22 benches durmientes, baselines placeholder |
| 18 | **desktop/** (Vanta Studio) | ~31,250 | **7.5** | 0 `any`, capabilities ejemplares, e2e reales; **el 100% del testing frontend fuera de CI** |
| — | **docs/** | 237,212 | **7.0** | Honestidad referente; pero burocracia que se auto-mide mal y 4.5K LOC de docs fantasma |
| — | **CI/CD + releases** | 26 workflows | **7.5** | Pinned por SHA, OIDC, Miri/fuzz/mutants; Fast Gate no corre en `develop` |
| — | **PRODUCTO** | — | **4.0** | GO-CON-CONDICIONES: embudo público roto en 3 peldaños; ver §7 |

**Los 5 hallazgos más graves (todos verificados de primera mano):**
1. 🔴 `POST /api/v2/export` e `/import` permiten **escritura/lectura arbitraria de archivos** (`handlers.rs:676-741`) — sin auth en modo dev.
2. 🔴 Proxy: `GET /snapshot` **sin autenticación** + bind `0.0.0.0` por defecto — expone sesiones, keys y costos.
3. 🔴 El instalador oficial de **macOS está roto** (`install.sh:119` usa `sha256sum`, inexistente en macOS).
4. 🔴 El script central de benchmarks Python está **roto desde agosto** (API renombrada) — la tabla §2 de BENCHMARKS.md es un fósil congelado y el README publica 2.0ms contra 62ms sin explicar (brecha 30×).
5. 🟠 Hooks de skills (SessionStart/PreCompact/Stop) **fallan en silencio en runtime** — protocolo JSON-RPC equivocado y pwsh como dependencia universal.

---

## 2. MAPA DEL REPOSITORIO

```
Vantadb/ (rama develop, 2,992 archivos, 45MB)
├── src/                    98,842 LOC — motor core (storage, HNSW/IVF/SCANN/DiskANN, BM25, WAL,
│   │                                   SDK Embedded, server HTTP axum, CLI, TUI, graphrag, parser IQL)
│   └── (98.5% vivo, ~600 LOC muertos identificados)
├── vanta-memory/           30,958 LOC — pipeline memoria L0→L3 (port TDAM), escenas, skills, dreams
├── vantadb-mcp/            20,512 LOC — servidor MCP stdio: 87 tools + resources + prompts
├── vanta-proxy/            14,160 LOC — proxy LLM: OpenAI/Anthropic/Responses, inyección memoria,
│                                       failover, redact, cache, rate-limit, cost
├── desktop/                31,250 LOC — Vanta Studio (Tauri v2 + React 19): 12 lentes, 3 transportes
├── vantadb-server/         1,930 LOC  — binario wrapper HTTP/MCP
├── vantadb-node/           3,586 LOC  — bindings napi-rs (0.5.0 STALE)
├── vantadb-python/         8,087 LOC  — bindings PyO3 (versión dinámica ✅)
├── vantadb-ts/             6,810 LOC  — SDK TypeScript sobre WASM
├── vantadb-wasm/           ~2,500 LOC — crate wasm-bindgen: OPFS/IndexedDB/Worker
├── providers/              2,274 LOC  — PyO3 openai/ollama/litellm (no publicados)
├── integrations/           ~2,500 LOC — 9 adapters Python puros (no publicados)
├── examples/               ~3,000 LOC — 10 py + 4 rust + 2 demos + Colab (roto)
├── benchmarks/             ~800 LOC   — bench Python (ROTO), competitive_bench (bueno)
├── benches/                4,242 LOC  — 22 benches criterion (13 durmientes)
├── embeddings/             ~600 LOC   — 9 modelos ONNX rev-pinned + verificación
├── docs/                   237,212 LOC — user/ dev/ api/ + 939 tasks + 98 planes + 52 ADRs
├── skills/                 ~1,500 LOC — 2 skills producto + hooks templates (rotos)
└── .github/workflows/      26 workflows — CI estratificado (Fast/Heavy/Informational)
```

---

## 3. ANÁLISIS PROFUNDO POR MÓDULO

### 3.1 Core engine — `src/` (98,842 LOC) · **8.5/10**

**Arquitectura:** storage/ (16,263) · sdk/ (16,111) · index/ (12,197: HNSW, IVF, SCANN, DiskANN, flat, kernels SIMD) · server/ (5,656) · cli_handlers/ (4,213) · metrics/ (2,582) · node/ (2,261) · parser/ + physical_plan/ + executor/ (5,081: DSL IQL) · resto (~9,000: graphrag, wiki, entity, vector/cuantización, TUI, shred, hardware).

**Fortalezas verificadas:**
- **7 `.unwrap()` / 1 `panic!` en ~99K LOC de producción** (1,767 unwraps en tests — separación limpia). Lints `unwrap_used/expect_used = deny` en prod.
- **0 TODO/FIXME/HACK** — convención propia `ponytail:` (86 marcadores con decisión + condición de upgrade).
- WAL con CRC32C por registro, auto-healing scan-forward, salvage con quarantine, verificación fail-closed de shards truncados.
- 51 `unsafe` concentrados en 3 módulos con SAFETY comments (vfile_mmap con handler SIGBUS propio, kernels SIMD con `chunks_exact(8)`).
- ~2,780 tests (2,307 inline + 473 integración), 24 benches, 26 proptest, failpoints + chaos harness.
- Orden de escritura WAL→storage→checkpoint correcto; no-reentrancia de flush resuelta con `LockPolicy::AssumeHeld` documentada.

**Fallas y riesgos de corrección:**
1. **126 líneas de comentarios con mojibake** (`ΓÇö`, `Ã©`) en 10+ archivos críticos (`storage/engine/insert.rs`, `txn.rs`, `get.rs`…) — corrompe la documentación de invariantes ACID.
2. **Durabilidad por defecto = `SyncMode::Periodic`** (`config.rs:256`): pérdida posible hasta `flush_threshold` en crash de SO (trade-off consciente de ADR-038, pero no surfaced al usuario).
3. **Escritura multi-store no atómica in-process** (documentada en `insert.rs:146-153`): WAL→File→KV; error a mitad deja estado parcial hasta compactación.
4. **Salvage de WAL sin fsync del truncado** (`wal_sharded.rs:262-285`): ventana de pérdida del quarantine.
5. **`engine.rs` "Fase 1" duplica semántica** del StorageEngine sin índices ni SIMD (búsqueda O(n) secuencial).
6. **Bloom filter público sin cablear** (`utils/duplicate_prevention.rs`): FPR real ~2% con k=3 subóptimo; API engañosa.
7. **`eviction.rs` completo (311 LOC) muerto**: 0 callers en todo el repo, feature `bayesian_decay` huérfana.
8. **`shred` delete-path no cableado**: entradas huérfanas hasta GC; el filtro puede leer columnas de nodos borrados.
9. **`io_budget` vaporware** (`executor.rs`): contador escrito, nadie lo lee; "hardware backpressure" prometida no existe.
10. Cast truncante u128→u64 en recovery (`engine.rs:209`); guard thread_local frágil si get/prefetch se vuelve async (`get.rs:26-70`).

**Optimizaciones concretas:**
- Clon de vector en staging batch (`insert.rs:96`) → mover; re-add HNSW clona bitset+vector (`maintenance.rs:176`) → `Arc<[f32]>`.
- `get()` clona `UnifiedNode` completo por lectura → API borrow/COW.
- Motor simple sin kernels SIMD (que ya existen) ni rayon.
- Visited-set de HNSW sin pool (el pool de `NeighborVec` ya existe).
- `lookup_int_le` O(#valores) sobre HashMap → `BTreeMap` para range-scan.
- Recovery scan-forward byte-a-byte → saltar por framing (len+CRC).

**Veredicto compilación:** `cargo check -p vantadb --all-targets` en Rust 1.95.0 → **0 errores, 0 warnings** (verificado en sandbox, 7m41s).

---

### 3.2 `src/` restante (19,458 LOC) · **8.0/10**

| Módulo | LOC | Wired a | Nota |
|---|---:|---|:---:|
| `entity/` | 1,892 | proxy, skills, server, wiki | 8.5 — DDD real, 981 LOC tests |
| `metrics/` | 2,582 | `/metrics`, `/api/v2/metrics`, desktop | 8.5 |
| `sdk/builder.rs` + `version_history.rs` | 967 | todo el SDK | 8.5 — `supersede_lock`, versionado honesto |
| `hardware/` + `vector/` (cuantización) | 1,881 | kernels, engine | 8.5 — miri-aware, mmap-safe |
| `agentic/` (threads) | 430 | MCP, proxy, server | 8.0 |
| `wiki/` | 1,358 | MCP wiki_*, vanta-memory | 8.0 — optimistic locking |
| `parser/executor/physical_plan` (IQL) | 5,081 | MCP, HTTP, CLI, SDK | 8.0 — fuzz target; io_budget vaporware |
| `backends/` (rocksdb/fjall/in_memory) | 1,432 | engine/init | 8.0 |
| `graphrag/` | 384 | SDK + 8 tools MCP | 7.5 — sin HTTP/CLI; `code_files` stub |
| `tui/` | 952 | `vanta-cli` (feature opcional) | 7.5 — REPL read-only testeado; Dashboard/Monitor sin tests |
| `shred/` | 703 | sdk memory + lexical | 7.0 — `allow(unwrap_used)` a nivel archivo; delete-path pendiente |
| `eviction.rs` | 311 | **nada** | **2.0 — código muerto con tests impecables** |

**Estimación global:** ~98.5% del core es código vivo conectado a una superficie real; ~600 LOC muertos con nombre y apellidos (`eviction.rs`, `duplicate_prevention.rs`, tier promotion de `lsm.rs`, io_budget).

---

### 3.3 `vanta-memory/` (30,958 LOC) · **7.5/10**

**Arquitectura L0→L3** (port de TDAM, todo persiste vía `vantadb::sdk::Embedded`, cero archivos propios):
- **L0** captura cruda idempotente (`l0/<session>` + cursor persistente, LLM-free, nunca bloquea).
- **L1** extracción + dedup en 2 fases (recall LLM-free + 1 llamada LLM que emite `store|update|merge|skip`); sin candidatos → degradación total a `store`.
- **L2** escenas: LLM propone, capa determinística decide UPDATE>MERGE>CREATE + heat + soft-delete (`[DELETED]`).
- **L3** persona con triggers P1-P4 sobre checkpoint persistido.
- **Post-L3** context assembly: compresión por cascada + inyección MMD bajo presupuesto compartido de tokens.
- **Dreams**: consolidación idle (≥10 min) que **jamás muta `l1/`** (invariante testado byte-identical); `promote` es stub preview-only.

**Calidad:** 2 unwraps prod (mock), 0 panics, 0 TODO; 563 tests deterministas (FakeClock, cero sleeps); documentación que traza cada archivo a su task MEM-xx; degradación LLM-free genuina y probada.

**Hallazgos críticos:**
1. 🔴 **`llm-driver` muerto en producción**: ningún consumidor habilita la feature; `StandaloneLlmRunner` sin ella retorna `LlmError::NotConfigured`. Resultado: en el binario MCP shipped, `VANTADB_INGEST_PROVIDER=ollama|openai` **silenciosamente no hace nada**. Fix: 1 línea en `vantadb-mcp/Cargo.toml`.
2. 🟠 **~40% del crate en pausa**: el scheduler completo (`PipelineWorker`, `MemoryPipelineManager`, `HttpCaptureBridge`, ~1,700 LOC — el subsistema mejor construido) no tiene ningún caller productivo (`bootstrap.rs:332` deja `conversation_trigger: None`). El ciclo automático L1→L3 solo corre bajo un host que no existe aún.
3. 🟠 Cola de aprobación (MEM-68) sin alimentador ni consumidor (FIND-110 DEFER — "mostrador vacío").
4. 🟡 Recall O(N) full-scan de namespaces `l1/*` (`auto_recall.rs:286` — ponytail correcto hoy, primer muro de escala).
5. 🟡 Tenancy default silenciosa: toda L1 nueva se estampa con team/agent default — puede sorprender (`l1_writer.rs:110-113`).
6. 🟡 Estado volátil del backend: cola L1, buffers y timers se pierden al reiniciar (mitigado: L0 persiste).
7. 🟡 Duplicación `MemoryPipelineManager` vs `StatefulPipelineManager`; `memory_prompt` (542 LOC) huérfano.

---

### 3.4 `vanta-proxy/` (14,160 LOC) · **7.5/10**

**Pipeline (16 etapas, orden impecable):** resolve_route (tier CC) → auth D34 → budget → guardrails → rate-limit → mem-command → session (TTL 30min, cap 10k) → inject (persona+escena en system, KV-cache-safe, idempotente byte-a-byte) → redact → context trim → translate → cache → forward con failover + tool-loop (max 3 iter) → capture/writeback fire-and-forget (3 reintentos + cola JSONL).

**Calidad de código sobresaliente:** **0 unwrap/expect/panic/unreachable en producción**; 12 ponytails; tests e2e de contrato con upstream mock real (~101 tests); clasificador de Claude Code por posición de `cache_control` con fallbacks conservadores.

**Hallazgos:**
1. 🔴 Seguridad ya reportada: `/snapshot` sin auth, `0.0.0.0` default, cache sin aislamiento por usuario, redact bypasseable sin header de sesión, sin body limit, `Authorization` reenviada a todos los upstreams del failover si falta api_key propia.
2. 🟠 **El loop de memoria es inerte**: `build_memory_block` lee persona/escenas que el proxy **nunca escribe** (no hay auto_capture ni promoción L0→L1); lo capturado con `vanta_memory_capture` va a `proxy-turns`… y `vanta_memory_search` lee `l1/{session}` — **namespaces disjuntos**: lo que el modelo captura nunca es recuperable por el propio proxy. En deployment proxy-only, la memoria autocontenida es ilusoria (bueno contra autoenvenenamiento, malo como promesa de producto).
3. 🟠 **Cost tracking fantasma**: `record_response_usage` **nunca se llama** (admitido en `server.rs:251`) → output tokens/costo de salida siempre 0; input estimado con chars/4 (±30%); ledger en memoria (restart = cero); precios stale (claude-3-5-sonnet).
4. 🟠 **Protocol break silencioso**: `stream:true` + translate Anthropic→OpenAI entrega chunks OpenAI crudos al cliente Anthropic (`openai_chunk_to_anthropic` es dead-code testado pero no cableado).
5. 🟡 Inyección **sin presupuesto de tokens** (persona larga entra entera en cada request; degrada prompt-cache).
6. 🟡 Timeout reqwest 600s es de duración total → corta streams largos legítimos; backoff sin jitter (documentado).
7. 🟡 Blocks `thinking` del stream → bloques `text` vacíos en reconstrucción de historial (riesgo multi-round con extended thinking + tools).
8. 🟡 Config TOML sin `deny_unknown_fields`: un typo se ignora silenciosamente.

---

### 3.5 `vantadb-server/` + capa HTTP del core · **8.0/10**

- **bootstrap.rs (refuse-to-start, FIND-07):** host no-loopback sin api_key → error accionable con 3 remedios; `is_loopback_host` fail-closed. **Ejemplar** — el proxy debería copiarlo.
- **RBAC 3 capas reales** (Bearer opaco constant-time + service-id + user-key deny-by-default); roles hardcodeados en `router.rs:135-137` (rígido); namespace-scoping solo en records/search/list — snapshots/maintenance quedan con permiso global.
- **JWT HS256 (ADR-039):** algoritmo pineado (anti alg-confusion), `exp` obligatorio con leeway 0, fallo → 401 genérico sin oráculo. Adecuado para MVP single-instance; rotación = env + restart (deuda asumida).
- **TLS** rustls 1.2+1.3 feature-gated; middleware en capas correcto (breaker → 1MB body limit → CORS → governor → auth → timeout); XFF solo de trusted proxies (5 tests).
- **37 paths / 44 operaciones** — `openapi.yaml` coincide **exactamente** (verificado con `check_openapi_parity.mjs`), pero el gate que vigila la paridad (`gate-docs.yml`) tiene el trigger corrupto → la paridad de hoy es fragilidad mañana.
- Dashboard HTTP: mount point correcto, **SPA inexistente en el repo** (404 con hint).
- docker-compose.prod.yml: hardening real (read_only, cap_drop, no-new-privileges); healthcheck de dev compose roto (usa `wget`, la imagen solo tiene `curl`); bind loopback + `ports:` publicado = puerto muerto (trampa operativa documentada).

---

### 3.6 `vantadb-mcp/` (20,512 LOC) · **8.0/10**

- **87 tools verificadas en código** (49 base + 38 extendidas) + meta-tests que **asertan las 87 exactas** y las annotations por tool. 229 tests sin un solo `#[ignore]`; 86/87 tools cubiertos (`rehydrate` el único huérfano).
- **Errores accionables de manual**: `dim_mismatch_guidance` con 2 caminos de recuperación; canal dual (JSON-RPC -32602 vs `isError` legible por el LLM); envelope ERR-MCP-01 `{code, retriable, hint}`; `skill_not_found` indistinguible de "no es tuyo" (anti side-channel).
- **MCP-35 fallback HTTP con ingenio**: axum efímero en `127.0.0.1:0` + discovery file con pid-guard; el segundo proceso ante `DatabaseBusy` hace proxy → stale cleanup → retry.
- **Problemas:**
  1. 🟠 **Default de perfil = `full`**: `tools/list` ≈ **90KB ≈ 22-25K tokens por sesión** solo para elegir tools. Existen perfiles (`memory` 22, `dev` 37-38) bien calibrados pero no son default.
  2. 🟠 **El perfil no se enforcea en `tools/call`** — una tool no listada sigue siendo llamable; el error `"(not in profile)"` que documenta `docs/api/MCP.md:254` **no existe en el código** (drift verificado).
  3. 🟠 Redundancias: `memory_search` alias de `search_memory` (deliberado, OK); `code_callers/callees/impact/explore` ≈ graph_traverse con otro formato (8 code_* aportan 1 primitiva real); triple solape de introspección (code_status ≈ operational_metrics ≈ capabilities).
  4. 🟡 `tools.rs` god-file (3,880 líneas) con **dos estilos de parsing** (la base no usa los helpers que sí usan los extendidos); duplicación server↔proxy justificada con un comentario **incorrecto** ("import cycle" imposible intra-crate).
  5. 🟡 Limitación honesta documentada: `spawn_blocking` no cancelable — N operaciones colgadas saturan el pool (MOD-11 H5).

**Recomendación de superficie:** 87 → ~65 reales (fusionar proyecciones, absorber alias) y **default a un perfil "agent" ~45** (memoria + threads + scenes + context + wiki-read); `full` para compatibilidad.

---

### 3.7 `skills/` + SKILLS-MANIFEST.md · **6.5/10**

- **Lo bueno:** `recall-policy.md` es referente (reglas estructurales con referencias file:line verificables); TOKEN-BUDGET.md con aritmética del 10%; `test-mcp.py` como drift-gate contra binario real (288 líneas, anti-stale); threat model LLM06 en SKILL.md de mcp.
- **Lo roto (verificado):**
  1. 🔴 **Hooks multi-cliente fallan en silencio en runtime**: el bridge JS envía `{"tool":"memory_recall"}` al stdin del launcher que arranca el servidor MCP **que espera JSON-RPC** → el servidor responde `-32700`, el hook lo parsea como error, `prepend_context` nunca aparece → **inyecta NOTHING**. La política anti-ruido "empty → inject NOTHING" **enmascara el bug a la perfección**. Además: `Array.isArray(hits)` sobre un valor que es `string|null`.
  2. 🔴 **Todo depende de `pwsh`** — incluso el plugin JS de OpenCode y los 4 configs de cliente. En macOS/Linux sin PowerShell: cero hooks.
  3. 🔴 Los hooks de Claude/Cursor/Codex lanzan el **servidor MCP long-running como hook command**: el stdout será el handshake JSON-RPC, no el schema de salida que el cliente espera (`additionalContext`).
  4. 🟠 `api-reference.md` ("single source of truth") dice **"86 tools" y omite 6 de las 87** — incluyendo `memory_recall`, la herramienta insignia de la propia recall-policy. El gate solo compara hashes entre mirrors, no contra el código → el drift pasa CI.
  5. 🟡 Versiones stale: `setup-vantadb.sh` pinea 0.5.0; SKILL.md de `vantadb` habla de "0.5.x boundaries".
- SKILLS-MANIFEST.md (602 líneas) **no es el manifiesto de las skills de producto** — es el catálogo de 196 skills personales del autor.

---

### 3.8 Bindings: `vantadb-node` · **8.0/10** · `vantadb-python` · **8.5/10** · `vantadb-ts` · **7.5/10**

**Paridad de API (matriz consolidada):** put/get/delete/flush/close paritarios. Gaps sistémicos:
- 🔴 **Text-only search imposible por la puerta principal de los 3 SDKs** (el core lo soporta: `query_vector: []` = solo-BM25) y **`query_sparse` no expuesto en ninguno** (se guarda sparse_vector pero no se puede consultar).
- 🟠 Formato de filtros **no intercambiable** py (`dict $eq/$gte`) vs js (`FilterItem[]`); catálogo de índices desalineado (`diskann` sin parsear en python).
- 🟠 `score` vs `distance`: **TS documenta al revés** — `distance: h.score` con doc "lower = more similar", falso para cosine (mayor=mejor). Induce a ordenar hits al revés. El mismo campo es distancia real en `searchVector`.
- 🟡 features repartidas asimétricamente: export/IQL/pagerank solo py o solo ts; `versions` solo node; `vacuum` solo node; 3 convenciones de node-ids (int u128 / decimal-string / number|bigint — hazard documentado).

**vantadb-node:** OpGate close-drain probado (carrera write-after-close cerrada), panics → napi::Error (no tumban Node), ids u128 seguros. Fallos: `distance_metric` desconocido → **Cosine silencioso** (py/TS dan error — paridad rota); vectores como `number[]` (doble copia vs Float32Array); `index.d.ts` commiteado stale (faltan 4 tipos del header — falla `tsc` si apuntas al repo); **versión 0.5.0** — nunca publicado al tren 0.6.x (tag `node-v*` que nadie corta); quickstart del README roto (`hit.hits[0]` sobre un array).

**vantadb-python:** el mejor ingenierado — 52 `py.detach` verificados uno a uno (drain sin GIL en close, razonamiento correcto y probado), batch con rayon, errores tipados ×11 con `.code/.retriable/.hint`, subclients vía macro sin duplicación, UAF NumPy histórico resuelto honestamente (a costa del zero-copy out), versión dinámica desde Cargo (fix del incidente PyPI 0.6.0). Gaps: alias `vantadb_py` prometía removal "en 0.6.0" y sigue (0.6.1).

**vantadb-ts:** validación de frontera la más estricta de los 3 (guards de namespace/vector/top_k/métrica); 6 casts `as unknown as` por drift del .d.ts hand-written; `importRecords` reimplementado sobre get+put loop por bug del binding WASM (FIND-79) — workaround O(N) round-trips; dualidad `Client` (sync) vs `NativeVantaDB` (async, 11 métodos) sin interfaz compartida; `engines >=22.19` restrictivo (razón documentada); README con `await` sobre API sync.

---

### 3.9 `vantadb-wasm/` · **8.0/10**

- Motor **InMemory por diseño** (no compila fjall); persistencia = snapshot JSON diferencial (`PersistCache` solo dirty/deleted; write fallido invalida la cache — nunca skip silencioso).
- **Durabilidad browser mejor de lo que su propia doc dice**: escritura atómica tmp+move con **CRC-32 footer verificado en lectura** (`opfs.rs:291-365`), pre-flight quota check vía `navigator.storage.estimate()` (bloquea >100%, warn 90%), `navigator.locks` para serializar writes IDB, auto-save opt-in con debounce 2s + best-effort en `pagehide` (WSM-03). **CRASH_MODEL.md está stale en 3 puntos** (dice "future" de cosas ya implementadas).
- OpGate adaptado a single-thread con cfg correcto (Condvar::wait panicaría en wasm32); rayon fuera; bundle 1.58MB / ~660KB gzip documentado sin humo; e2e real con Playwright + reload (Chrome, OPFS e IDB).
- MAX_RECORDS = 1,000,000; export a filesystem no viable en browser (gap menor).

---

### 3.10 `providers/` (2,274 LOC) · **7.5/10** · `integrations/` (~2,500 LOC) · **7.0/10**

- **providers** (PyO3): error handling espejo MOD-20, `shared_py.rs` anti-drift, `.pyi` verificados en CI. Pero: **no publicados en PyPI** (`publish=false`, sin pyproject — consumo = clonar + maturin), versiones 0.5.0 stale, ~60% copy-paste entre los 3 (difieren solo en la llamada embed), **colisión de nombres con integrations**: ambos producen el módulo `vantadb_openai` con `VantaDBOpenAI` de constructor **distinto e incompatible**.
- **integrations** (9 adapters Python puros: langchain, llamaindex, mem0, crewai, dspy, haystack, letta, openai, ollama): tests reales, gate de pins en CI (con fragilidad: fallback instala literalmente `"0.3"`; pin ollama 0.3 < floor 0.4 del adapter). Deuda: importan `vantadb_py` (deprecado — time-bomb), **nunca publicados** (el workflow existe; jamás se taggeó `adapters-v*`), README triple-stale, convención score abierta.
- Matriz "18 combinaciones" de adapters-compat.yml = 9 adapters × 2 versiones; **litellm ausente**.

---

### 3.11 `examples/` · **6.0/10**

| Ejemplo | Estado |
|---|---|
| `rag_pdf_chat` (SHOW-03) | ⭐ Sobresaliente — genera su propio PDF fixture, determinista, assert de cita |
| `agent_memory_cli` (SHOW-04) | ⭐ Sobresaliente — 2 sesiones, asserts de contenido exacto |
| `rust/` ×4 | Bien — API actual, `Box<dyn Error>`; dejan `examples_*_data` sin limpiar |
| `demo/demo.py` | Bien — fallback sentence-transformers |
| `langchain_ollama_rag.py` | El mejor python — usa integración real, degrada a mock |
| 6 ejemplos framework (autogen, crewai, dspy, haystack, mem0, semantic_kernel, langgraph) | Esqueleto CRUD idéntico que **emula** la API del framework sin importarlo; solapan con `integrations/*` |
| **`colab/vantadb_quickstart.ipynb`** | 🔴 **ROTO** — usa `vanta.VantaDB`, `get_memory`, `search_memory` (API removida en 0.5.0) → `AttributeError` garantizado. Es la puerta de entrada del badge del README |
| Tests huérfanos | 🟠 `test_agent_memory_cli.py` y `test_rag_pdf_chat.py` (los mejores del árbol) **no corren en ningún workflow** (grep en `.github/` = 0) |
| README examples | Miente ×2: referencia `ci-examples-12.yml` (no existe) y afirma "todos vigentes en CI" (falso para el notebook) |

---

### 3.12 `embeddings/` · **8.0/10**

- Manifest con **9 modelos rev-pinned** (7-hex) en 3 grupos (EN/ES/combined): `multilingual-e5-small` default (384d, 220MB, MIT, prefijos `query:/passage:` medidos), bge-mini, MiniLM, jina-es, bge-m3 (1.2GB int8), y la excepción `qwen3-embedding-8b` (4096d, 16GB, GPU, `--skip-exception`).
- `download.py` con ALLOW_PATTERNS recortado (evitaba descarga 3-4× duplicada), `manifest.lock` acumulativo commiteable, `--check` offline, verificación en 2 niveles (smoke ONNX + numérica en `sanity_embed.py`).
- Regla de oro una-dim-por-base con error accionable (EMB-18). Cache local ~22GB los 9 (gitignored, no gestionada por el repo).

---

### 3.13 `benchmarks/` (Python) · **3.5/10** · `benches/` (criterion) · **8.5/10**

**benchmarks/ — el talón de Aquiles público:**
- 🔴 `vantadb_local_bench.py` está **ROTO desde el rename 0.5.0** (usa `vantadb.VantaDB(...)` / `db.search_memory(...)` — métodos removidos; sin commit desde 2026-08-02). `perf-bench.yml` **falla en su step principal en cada corrida**.
- `python_baseline.json` vacío desde 2026-08-12; `criterion_baseline.json` con placeholders nominales "derived from inspection" → el gate anti-regresión del 15% es **doblemente no-op**.
- `update_markdown.py`: efecto nulo triple (no commitea, tabla en español vs doc en inglés, depende del script roto).
- `download_benchmark_datasets.sh`: GloVe sin checksum.
- Punto alto: `competitive_bench.py` (LanceDB/Chroma/Qdrant/Milvus, recall honesto, "honesty contract", seed 42).

**benches/ — metodología verificada al detalle:** `common/mod.rs` cumple exactamente lo declarado (warm_up 3s, measurement 5s, confidence 0.95, noise 0.05); dataset sintético **commiteado y determinista** (xorshift64*, 2000×256 f32); `wal_throughput` con `iter_custom` + tempdir fresco (fsync frío comparable); `canonical_p99` a 100k×1536 como contrato anti-regresión. Gaps: **solo 9 de 22 corren en el nightly** (13 durmientes sin señal); `high_density` usa `rand::rng()` (único no-determinista); `list_window.rs` registrado en ninguna parte (bench huérfano).

**Causa raíz de la brecha 30× en números públicos (verificada con git):**
1. Tres pipelines miden cosas distintas sin etiquetarlo: §1 criterion Rust (1.2ms @10K, AVX2, Ryzen 12c), §2 SDK Python en runner CI compartido (61.99ms), README = artifact gitignoreado local de Windows (2.0ms).
2. **El generador de §2 murió** con el rename de API → tabla congelada en la era pre-optimización.
3. El motor mejoró 20-50× después (Phase 2: cache O(M²), SIMD f32x8, 3,636 QPS) — el README tomó su 2.0ms de una corrida local reciente; §2 nunca se regeneró.
4. Factores secundarios: frontera PyO3+GIL (9.73ms secuencial vs 2.43ms batch), runners CI sin garantía AVX2, mmap frío.
- **Conclusión:** 61.99ms es un fósil; 2.0ms es el motor actual; 1.2ms el techo Rust. La brecha ≈ engine-viejo-congelado × pipeline-distinto × hardware-distinto publicado como si fuera comparable. **La ingesta 74 rec/s sí es real** y su cuello es `insert_lock` global + HNSW serial (98.5%; fsync solo 1.5% — FIND-61); el batching prototipado alcanza **1,016 ops/s (9.1×)** y no está productizado (FUT-12).

---

### 3.14 `desktop/` — Vanta Studio (~31,250 LOC) · **7.5/10**

**Fortalezas:** transporte de 3 backends bien resuelto (Tauri IPC / HTTP / WASM+OPFS con degradación explícita); **0 `any` en todo src/**, strict TS; capabilities de Tauri con mínimo privilegio ejemplar (solo core + 4 de ventana); 6 stores vanilla propios con sanitización de JSON corrupto; undo persistente (MAX_HISTORY=50); i18n ES/EN con paridad testeada; a11y seria (focus-trap, aria, prefers-reduced-motion, auditoría WCAG AA documentada); e2e Playwright **genuinos** (compilan dist-web, arrancan server real, siembran por REST, recorren ingest→inspector→papelera→restore por teclado, coleccionan pageerror); deep-links tratados como input no confiable.

**Fallas:**
1. 🟠 **El 100% del testing frontend está fuera de CI** — `desktop.yml` compila sidecars pero **no corre ni vitest ni Playwright** (verificado grepeando todos los workflows). 14 suites vitest + 9 node-test + 6 e2e dependen de disciplina local. Conectarlo es barato (los sidecars ya se compilan ahí).
2. 🟠 `"build": "vite build"` **sin `tsc`** (README dice `tsc && vite build`); tests excluidos del tsconfig → errores de tipo pueden llegar a producción.
3. 🟠 CSP con `connect-src https://*` — permite fetch a cualquier host desde el WebView (amplifica un XSS hipotético; no es necesario para la función actual).
4. 🟡 `WorkspaceShell.tsx` monolito de 1,206 líneas (sidebar+topbar+router de 12 superficies+6 atajos+4 diálogos); `DataExplorer.tsx` 974; `ConsolidateLens` 637.
5. 🟡 Smells de producción Rust: `Embedded::test_empty` en `commands/metrics.rs:30`; `vanta_health` crea una DB desechable en temp **por cada probe sin borrarla**.
6. 🟡 Tokens Bearer en localStorage (sin keychain del SO); `vanta_export_namespace` escribe en ruta arbitraria que pase el frontend.
7. 🟡 3 runners de test (vitest + node --test + 7 selfchecks legacy) superpuestos; i18n a medias (slice 3 DEFER); bundles sin firmar (sin certificado Apple/Windows — admitido).
8. 🟡 Wire duplicado: `vanta.ts` replica a mano ~30 interfaces de `types.rs` sin verificación de contrato compartida (ts-rs/specta lo resolvería).

---

### 3.15 `docs/` (237,212 LOC) · **7.0/10**

**Fortalezas:** docs/api/ es la mejor superficie documental (openapi.yaml con paridad exacta verificada; SDK docs por lenguaje; scores.md espejo del contrato canónico); book mdBook sin duplicación (`{{#include}}`); glosario de 63 términos; currículo de aprendizaje sobresaliente; QUICKSTART verificado end-to-end por "test usuario real" el 2026-09-23 (R-05); honestidad excepcional (COMPARISON con "What We Deliberately Do Not Claim", INV-013 marcando un claim propio como FALSO).

**Fallas:**
1. 🔴 `docs/CHANGELOG.md` corrupto: **dos changelogs concatenados** (2× `# Changelog`, 2× `## [Unreleased]`, frontmatter cosido en línea ~1836).
2. 🔴 `gate-docs.yml` trigger corrupto (`branches: ain, develop]`) → markdownlint + paridad openapi **sin vigilancia en push/PR**.
3. 🟠 `docs/user/web/` = **4,528 LOC de documentación fantasma** ("status: active") de un frontend Next.js que se movió a repo propio el 2026-09-22.
4. 🟠 ADR-041 duplicado (anti_stutter proposed vs error-variant-renames accepted — el README cita el no-aceptado); ADR-021 vs ADR-025 contradictorios sin supersession; 3 convenciones de numeración.
5. 🟠 Cementerio: 939 archivos de task con 604 completadas sin archivar; IDs reutilizados (FIND-63 = 3 bugs distintos); contadores stale (Backlog declara "70 abiertas" con 109 filas reales).
6. 🟡 MCP.md se contradice internamente (línea 12 "0.6.1" vs línea 512 "0.5.0 as of this doc"); TRIGGERS.md documenta el estado deseado del trigger corrupto; graphrag README overclaima "LLM-powered Semantic Compression Engine" como primitiva core (vive en vanta-memory).
7. 🟡 README_ES con drift (badge Colab `main` vs `develop`; omite fila de examples y nota de naming); 7 blogs con `version: 0.5.0`; MCP_REGISTRY.md y hardening.md en 0.5.0.
8. 🟡 Versiones stale cruzadas: `vantadb-node/package.json` 0.5.0, providers 0.5.0, llms.txt 0.5.0 (y promete features que el boundary no reivindica), CITATION.cff 0.5.0, Homebrew 0.5.0, Dockerfile APP_VERSION 0.5.0.
9. ✅ 38/38 links relativos del README existen.

---

### 3.16 CI/CD + releases · **7.5/10**

**Lo excepcional:** 26 workflows con 100% de acciones pinned por SHA; **0 `pull_request_target`** (vector clásico ausente); OIDC trusted publishing (PyPI/npm) con **attestations verificadas** (`gh attestation verify`); ci-gate reusable fail-closed; estratificación real Fast/Heavy/Informational con taxonomía documentada; Miri + Tree Borrows, ASan/TSan, fuzz semanal, cargo-mutants, semver-checks, ADR Gate mecánico (PR que cambia API sin ADR = fallo); coverage gate ≥80%; 5 targets de binarios + sha256 sidecars.

**Fallas:**
1. 🔴 **El Fast Gate Rust no corre en `develop`** (`ci-rust.yml` solo main) — fmt/clippy/tests/coverage/Miri no ejercitan el 90% del desarrollo diario.
2. 🔴 `install.sh:119` usa `sha256sum` → **instalador macOS roto**; checksum fail-open si falta el asset (TOFU documentado).
3. 🟠 El wizard encadenado no puede funcionar desde el instalador (baja `setup-embeddings.ps1` a tmpdir donde `$PSScriptRoot` ya no resuelve el manifest).
4. 🟠 `opencode.yml`: agente AI por comentario de **cualquier usuario** con `id-token: write` — sin check de `author_association`.
5. 🟠 Baselines de regresión vacíos/placeholder (ver 3.13); colisión de crons domingo 03:00 (adapters-compat + heavy-certification).
6. 🟡 `release-sbom.yml` sube artifact `sbom-web.json` que nadie genera (`web/` ya no existe); `perf-bench` usa tamaños distintos por trigger (no comparables); `gate-docs` con `npx markdownlint-cli2` sin versión fija.
7. 🟡 Homebrew: script de update con ruta rota (`../vantadb.rb` vs `Formula/vantadb.rb`), fórmula stale 0.5.0, FAQ promete `brew install vantadb` (falso); sin firma criptográfica de binarios (cosign/minisign ausentes); Dockerfile: `cargo-watch` en builder prod, sin `--locked`, APP_VERSION stale.

---

## 4. CONSOLIDADO DE ERRORES Y FALLAS (por severidad)

### 🔴 CRÍTICOS (12)
| # | Hallazgo | Ubicación | Impacto |
|---|---|---|---|
| C1 | Escritura arbitraria de archivos vía `POST /api/v2/export` | `src/server/handlers.rs:676-714` | Compromiso total del host; sin auth en dev |
| C2 | Lectura arbitraria vía `POST /api/v2/import` | `src/server/handlers.rs:687-741` | Exfiltración de archivos |
| C3 | Path traversal en `POST /api/v2/snapshots/{name}` | `handlers.rs:1514-1529` | Escape del data dir (el MCP sí valida — doble estándar) |
| C4 | Proxy `GET /snapshot` sin auth + bind `0.0.0.0` default | `vanta-proxy/src/server.rs:744,816`, `config.rs:199` | Enumeración de sesiones/keys/costos por tercero |
| C5 | Instalador macOS roto (`sha256sum`) | `scripts/install.sh:119` | La promesa "1 comando" falla en todo Mac |
| C6 | Script de benchmarks Python roto (API renombrada) | `benchmarks/vantadb_local_bench.py` | Gate de regresión no-op; tabla §2 congelada; brecha 30× |
| C7 | `gate-docs.yml` trigger YAML corrupto | `.github/workflows/gate-docs.yml:6,9` | markdownlint + paridad openapi sin vigilancia |
| C8 | Fast Gate Rust sin trigger en `develop` | `ci-rust.yml:4-39` | El 90% del desarrollo sin fmt/clippy/tests |
| C9 | Hooks de skills rotos en runtime (protocolo + pwsh) | `skills/vantadb-mcp/assets/` | Recall automático inyecta NOTHING, en silencio |
| C10 | `docs/CHANGELOG.md` con 2 changelogs concatenados | `docs/CHANGELOG.md:~1836` | Changelog ilegible; riesgo para release-plz |
| C11 | `llm-driver` muerto: config LLM del ingest decorativa en MCP shipped | `vantadb-mcp/Cargo.toml:14` | `VANTADB_INGEST_PROVIDER=ollama/openai` no hace nada |
| C12 | Colab notebook con API removida (0.5.0) | `examples/colab/vantadb_quickstart.ipynb` | Puerta de entrada del README → `AttributeError` |

### 🟠 ALTOS (14)
| # | Hallazgo | Ubicación |
|---|---|---|
| A1 | Cache del proxy sin aislamiento por usuario/sesión | `vanta-proxy/src/cache.rs:157-178` |
| A2 | `api-reference.md` omite 6/87 tools (incluida `memory_recall`) | `skills/vantadb-mcp/references/` |
| A3 | MCP: perfil no enforceado en `tools/call` + doc de comportamiento inexistente | `vantadb-mcp` + `docs/api/MCP.md:254` |
| A4 | Proxy: loop de memoria inerte (capture→`proxy-turns`, search→`l1/` — disjuntos) | `vanta-proxy` inject/capture |
| A5 | Proxy: cost output tokens nunca cableado (`record_response_usage` sin caller) | `vanta-proxy/src/server.rs:251` |
| A6 | Proxy: `stream:true`+translate rompe el protocolo Anthropic | `translate.rs:416,456` (dead-code) |
| A7 | TS SDK: `distance` documentado al revés para cosine | `vantadb.ts:617`, `types.ts:89-95` |
| A8 | CSP desktop `connect-src https://*` | `desktop/src-tauri/tauri.conf.json:27,33` |
| A9 | Desktop CI sin vitest ni Playwright; build sin `tsc` | `.github/workflows/desktop.yml`, `package.json:8` |
| A10 | `opencode.yml` sin check de `author_association` | `.github/workflows/opencode.yml:17-38` |
| A11 | Baselines de perf vacíos/placeholder (gates dormidos) | `benchmarks/*_baseline.json` |
| A12 | `vantadb-node` 0.5.0 nunca publicado al tren 0.6.x (404 en npm) | `release-npm-node.yml` + tag `node-v*` |
| A13 | `vantadb_openai`/`vantadb_ollama` colisión de nombres providers vs integrations | `providers/` vs `integrations/` |
| A14 | vanta-memory: scheduler ~1,700 LOC (~40% del crate) sin host productivo | `bootstrap.rs:332` (`conversation_trigger: None`) |

### 🟡 MEDIOS (22)
SyncMode::Periodic default sin surfaced · escritura multi-store no atómica in-process · salvage WAL sin fsync · mojibake 126 líneas · engine.rs fase-1 duplicado · bloom filter FPR 2% sin cablear · eviction.rs muerto (311 LOC) · shred delete-path no cableado · io_budget vaporware · inyección proxy sin presupuesto de tokens · timeout 600s corta streams · config proxy sin deny_unknown_fields · RBAC roles hardcodeados + scoping parcial · text-only search y query_sparse no expuestos en 3 SDKs · formato de filtros no intercambiable py/js · node: distance_metric→Cosine silencioso · node: index.d.ts stale falla tsc · CRASH_MODEL.md stale ×3 · docs/user/web/ 4.5K LOC fantasma · ADR-041 duplicado / ADR-021↔025 contradictorios · versiones stale cruzadas (node, homebrew, llms.txt, CITATION, Dockerfile, 7 blogs) · healthcheck compose wget/curl.

### ⚪ BAJOS (15+)
Clamping con `eprintln!` · dashboard SPA fantasma · notebook/tests duplicados en examples · 6 esqueletos emulando frameworks · GloVe sin checksum · benches durmientes 13/22 + huérfano + no-determinista · `update_markdown` no-op · sbom-web artifact muerto · cron colisionado · `npx markdownlint` sin pin · cargo-watch en builder prod · tokens Bearer en localStorage · `test_empty` en metrics.rs prod · temp DB por health probe · i18n slice 3 pendiente · 3 runners de test superpuestos · fail-open generalizado del proxy (documentado) · checksum fail-open instaladores · CRE checks: README_ES drift.

---

## 5. OPTIMIZACIONES DE RENDIMIENTO (priorizadas por impacto)

1. **Productizar el batch insert (FUT-12/ADR-038)** — el prototype ya alcanzó **1,016 ops/s (9.1×)**; el techo actual (74 rec/s) es `insert_lock` global + HNSW serial (98.5% del tiempo; fsync solo 1.5%). Es el gap competitivo #1 (598 vs 114,583 ingest-QPS de LanceDB).
2. Kernels SIMD (`f32x8/f32x16`, ya existen) + rayon en el motor "Fase 1" (`engine.rs`) o eliminarlo.
3. `Arc<[f32]>` compartido en re-add HNSW y staging de batch (eliminar clones de vectores completos en hot paths: `insert.rs:96`, `maintenance.rs:176`).
4. API borrow/COW para `get()` (hoy clona payload+vector+edges por lectura).
5. Pool de visited-sets en HNSW (el pool de `NeighborVec` ya existe como patrón).
6. `BTreeMap` en `scalar_index.rs` para range-scan O(log v + k).
7. Recovery por framing (len+CRC) en vez de scan byte-a-byte.
8. Presupuesto de tokens del bloque `<vanta-memory>` del proxy (protege prompt-cache del upstream).
9. `Float32Array` en la frontera napi de node (evita doble conversión).
10. Parallelizar flat search con rayon sobre el threshold.

---

## 6. MEJORAS PRIORIZADAS (consolidado final)

### P0 — antes de cualquier marketing (1-2 semanas de trabajo concentrado)
1. Sandbox de export/import/snapshots HTTP a directorio base + validar nombre (paridad con el guard del MCP) → C1-C3.
2. Auth en `GET /snapshot` del proxy + refuse-to-start si bind no-loopback sin key (copiar FIND-07 del server) → C4.
3. Fix `install.sh` macOS (`command -v sha256sum || shasum -a 256`) → C5.
4. Arreglar trigger de `gate-docs.yml` (`[main, develop]`) → C7.
5. Reparar `docs/CHANGELOG.md` (deduplicar; verificar parser de release-plz tras la cirugía) → C10.
6. Portar el bench Python a `Client` (o enterrarlo junto con perf-bench.yml) + seed + warmup + metadatos de entorno → C6.
7. Reparar el notebook Colab → C12.
8. Forward `llm-driver` en `vantadb-mcp/Cargo.toml` (1 línea) → C11.
9. Reescribir el bridge de hooks sin pwsh, con un binario one-shot `vanta-cli mcp-call` (toda la maquinaria ya existe) → C9.
10. Extender `ci-rust.yml` a `develop` (o documentar trunk-based) → C8.
11. Regenerar §2 de BENCHMARKS.md y reconciliar README↔§1↔§2 (mismo dataset/máquina/versión, hardware normalizado); hacer que `update_markdown.py` produzca PR automático.
12. BND-07: nuevo invite de Discord + DNS de `vantadb.dev` (o migrar homepages) → actualiza README/SUPPORT/SECURITY.
13. Sincronizar versiones: publicar `vantadb-node` 0.6.1 (o gate CI de alineación), llms.txt, CITATION.cff, Homebrew, Dockerfile, skills 0.5.0.
14. Desktop: `"build": "tsc -b && vite build"` + conectar vitest/Playwright a `desktop.yml` + restringir CSP.
15. Regenerar `api-reference.md` (86→87) + test de nombres documentados vs `tools/list`.

### P1 — primer trimestre
16. Productizar batch insert (P0 de performance) y actualizar README con el gap competitivo calificado.
17. Cerrar el loop capture→search del proxy (escribir capturas en `l1/{session}` o buscar también en `proxy-turns`) + presupuesto de tokens de inyección.
18. Cablear `record_response_usage` (cost real) o degradar el claim de "cost tracking".
19. Cablear o rechazar explícitamente translate+`stream:true` (400).
20. Enforce de perfil MCP en `tools/call`; default a perfil "agent" (~45 tools); fusionar proyecciones (87→~65).
21. Exponer `query_sparse` y text-only search en los 3 SDKs (mayor gap de paridad real).
22. Alinear versión/registro de `vantadb-node`; publicar `integrations/*` a PyPI (tag `adapters-v*`); resolver colisión providers↔integrations.
23. Poblar baselines de perf y quitar fail-open; señal semanal para los 13 benches durmientes.
24. Wire del scheduler de vanta-memory: `HttpCaptureBridge` al ServerState + drenador, o exponer `scheduler_run_once` como tool MCP (diseño FIND-113), o congelar documentado.
25. Firmar binarios (cosign/minisign); Homebrew 0.6.1 automatizado post-release.
26. Cablear los 2 tests estrella huérfanos a `ci-examples.yml`; activar gates de regresión.
27. Archivar 604 tasks completadas + docs/user/web/ (4.5K LOC) + resolver ADR-041 duplicado; contadores de Backlog scriptados (GOV-C7).
28. Partir `WorkspaceShell.tsx` (<300 líneas); consolidar a 1 runner de test; DTOs TS generados desde Rust (ts-rs).
29. Eliminar código muerto core (`eviction.rs`, `duplicate_prevention.rs`) o moverlo a `experimental/`.
30. Decidir destino de `shred` delete-path y de `io_budget` (implementar o reformular como contrato documental).

### P2 — después
31. EXE-02: head-to-head Mem0/Zep/Letta con LoCoMo/LongMemEval (harness MEM-70 ya existe) — publicar win/loss.
32. MKT-04: publicar los 3 posts de Reddit; quitar `draft` de los 9 blogs.
33. Dictamen de marca "Vanta" **antes** de cualquier filing de LEG-01 (ver §7.4).
34. Desduplicar ejemplos (`_common.py` + convergencia sobre `integrations/*`); limpiar mojibake + gate de encoding.
35. Dashboard SPA mínima o documentar el build externo; RBAC desde config; graphrag a HTTP o declarado MCP-only.
36. Promover dreams `promote_dream_run` (merge real a L1); consolidar managers duplicados de vanta-memory; recall O(N) → índice.

---

## 7. ANÁLISIS DE PRODUCTO

### 7.1 Propuesta de valor y posicionamiento
- **Promesa consistente:** "SQLite para agentes de IA" — motor embebido, local-first, duradero (WAL crash-safe), híbrido nativo (BM25+HNSW+RRF), cero servicios externos, expuesto idénticamente a Python/TS/WASM/MCP.
- **Diferenciador real y defendible:** la **intersección única** embebido + híbrido + grafo/PageRank/GraphRAG + MCP profundo (87 tools, la más profunda del nicho) + Desktop Studio (único en la categoría) + embeddings locales ES/EN + proxy de redacción local. **Nadie más ofrece esa intersección.**
- **Fisuras del mensaje (6):** llms.txt stale (0.5.0) y prometiendo features que el boundary no reivindica; ICP sin decidir (BIZ-08 abierta — SPEC dice "agentes de código", GTM define 3 verticales); README destaca 74 rec/s sin contexto de batch; 9 blogs draft; COMPARISON compite contra vector DBs y **no contra Mem0/Zep/Letta** (los rivales reales por el mismo wallet); números no reproducibles como artifact.

### 7.2 Competencia
| | VantaDB | Mem0 | Zep/Graphiti | Letta | LanceDB | sqlite-vec |
|---|---|---|---|---|---|---|
| Categoría | Motor embebido + memoria | Memoria-as-a-service | Memoria temporal KG | Framework con memoria | Vector embebido columnar | Extensión SQLite |
| Financiación | 0 (1 persona) | **$24M+** | Series A | **$10M** | $30M+ | Indie |
| Embedded/local | ✅ core | ❌ | ❌ | ❌ | ✅ | ✅ |
| Híbrido nativo RRF | ✅ | ⚠️ | ⚠️ | ❌ | ✅ | ❌ |
| Grafo/GraphRAG | ✅ | ❌ | ✅ fuerte | ⚠️ | ❌ | ❌ |
| MCP profundo | ✅ 87 tools | ✅ básico | ✅ básico | ✅ | ❌ | ❌ |
| Desktop Studio | ✅ único | ❌ | ❌ | ❌ | ⚠️ pago | ❌ |
| Performance | pierde ingesta ~190×; query p99 competitivo | n/a | n/a | n/a | líder | 🟡 |
| Riesgo | — | Alto (mismo buyer) | Alto nicho | Alto (define categoría) | Bajo-medio | **Medio-alto (gratis cubre el caso simple)** |

**Conclusión competitiva:** no ganar en velocidad bruta ni ecosistema; el terreno defendible es la intersección + rigor de durabilidad + gobernanza de memoria (dedup/supersession/dreams/skills). El pitch debe migrar de "vector DB" a **"la memoria de agente que vive en tu proceso, con gobernanza y números verificables"**. La ventana es corta: Mem0/Letta avanzan hacia local/embedded.

### 7.3 Estado GTM — checklist de lanzamiento
| Canal | Estado |
|---|---|
| PyPI `vantadb-py` 0.6.1 | ✅ |
| npm `vantadb` (TS) 0.6.1 | ✅ |
| npm `vantadb-node` | ❌ nunca publicado (404) |
| crates.io | ⚠️ |
| Binarios | ⚠️ sin firma; install.sh macOS roto |
| Homebrew | ❌ stale 0.5.0 + script roto |
| Docker | ⚠️ build-no-push |
| Colab | ✅ badge — pero notebook roto |
| Sitio web | ❌ `vantadb.dev` sin DNS |
| Discord | ❌ invite inválido |
| Blog | ❌ 9 drafts |
| Reddit | ⚠️ 3 posts listos sin publicar |
| Adapters PyPI | ⚠️ code-done, sin taggear |
| security@/enterprise@ | ❌ dominio muerto |

### 7.4 Modelo de negocio y riesgos legales
- **Open Core (ADR-013) coherente:** core Apache-2.0 intocable; `vantadb-pro` propietario (repo privado con `license.rs` + verificación offline, sin call-home); Pro = features nuevas (RBAC multi-tenancy, replicación, PITR, admin), nunca features movidas del core. Pricing en papel (Cloud $99/$499; on-prem $10-50K/año). CLAs bien redactados pero **fricción innecesaria con 0 contribuidores externos** (DCO sería mejor hoy).
- ⚠️ **Conflicto de marca "Vanta" (riesgo serio y asimétrico):** Vanta Inc. (vanta.com) es un unicornio de compliance (~$2.4B) con marcas VANTA registradas en clases 9/42; ambos venden software a developers. El sufijo "DB" es diferenciador débil. El riesgo llega exactamente cuando el producto empiece a valer algo — justo cuando rebrandear es más caro. **Hacer el filing de LEG-01 sin dictamen puede atraer la atención que hoy no existe.** Recomendación: consulta formal de marcas ($500-1,500) antes de invertir un dólar en marca; preparar nombre de respaldo para la capa comercial.

### 7.5 Riesgos top 8
1. 🔴 Brecha de ingesta (191×) — mata el caso "ingesta continua de agente"; el fix (batching 9.1×) está prototipado y no productizado.
2. 🔴 Seguridad de endpoints en un producto cuyo argumento de venta es la privacidad — un solo CVE anula la narrativa local-first.
3. 🔴 Competidores financiados + sqlite-vec gratis cubriendo el caso simple.
4. 🔴 Bus factor = 1 humano + agentes: 939 tasks/150 planes/196 skills de proceso para 0 usuarios externos — el proceso consumirá al mantenedor antes que el mercado.
5. 🟠 Credibilidad de números: brecha 30×, JSON gitignored, baselines vacíos — en HN/Reddit los números son lo primero que se ataca.
6. 🟠 Distribución frágil: la promesa "1 comando" falla en macOS; binarios sin firmar; homebrew/node stale.
7. 🟠 Marca sin registrar + conflicto Vanta.
8. 🟠 Superficie social muerta (Discord, dominio, emails) — señales de abandono visibles en <5 min de visita.

### 7.6 SWOT
| **Fortalezas** | **Debilidades** |
|---|---|
| Durabilidad de élite (WAL auto-heal, chaos, Miri, 2,780 tests, lints anti-unwrap) | Ingesta 191× más lenta; techo arquitectónico conocido |
| MCP 87 tools — la superficie más profunda del nicho | 1 persona; proceso documental insostenible |
| Stack completo: core + 4 SDKs + REST + MCP + proxy + desktop | Marca débil: dominio/Discord muertos, conflicto Vanta |
| Embeddings locales ES/EN + honestidad de fallbacks | Números públicos no reproducibles-committed |
| Cultura de honestidad (Regla 11) — activo de marca real | Adapters listos y no publicados; marketing redactado y no publicado |
| Open-core bien diseñado (ADR-013) | Embudo público roto en sus 3 primeros peldaños |

| **Oportunidades** | **Amenazas** |
|---|---|
| MCP como canal único (1 integración → N clientes: Cursor/Claude/Codex/OpenCode/Cline) | Mem0/Letta/Zep capitalizan "agent memory" y bajan a local |
| Dolor real y creciente que SPEC describe (autoenvenenamiento, deriva, Lost-in-the-Middle) | sqlite-vec+FTS5 gratis satisface el caso simple |
| Nicho dev-tooling/privacidad sin competidor con desktop ni proxy de redacción local | Un CVE en export/import o proxy anula la propuesta |
| Head-to-head LoCoMo/LongMemEval con harness propio: historia publicable y verificable | Breaking changes frecuentes queman early adopters; SLA de 48h sin Core Team es humo |

### 7.7 Veredicto de preparación para lanzamiento: **4/10 — GO-CON-CONDICIONES**
- **NO-GO hoy** para Show HN/Reddit/anuncio público: el embudo está roto (install macOS, Discord, dominio), los endpoints de seguridad expuestos contradicen la propuesta de privacidad, y el gate auto-impuesto (EXE-03: "todo SÍ o no se anuncia") no se ha ejecutado.
- **GO** cuando el checklist P0 esté verde — estimación realista **2-4 semanas** de trabajo concentrado (la mayoría son fixes pequeños; lo más lento es EXE-03 con 5 testers humanos y el DNS).
- Condiciones no negociables antes de monetizar: BIZ-04 (ToS/Privacy) y dictamen de marca.
- El mejor momento del anuncio: tras EXE-03 verde + EXE-02 (head-to-head) publicado — pasar de "otro vector DB más lento" a "la memoria local más profunda para agentes, con números verificables" es la única historia ganable desde 1 persona sin financiación.
- **North Star sugerida:** agentes activos que recuperan una memoria con éxito en ventana de 7 días (proxy medible: sesiones MCP con put+search la misma semana). Guardrails: 0 hallazgos high sin parche ≤7 días; 0 regresión p99 >15% (gate revivido); 100% de artefactos con versión sincronizada; violaciones Regla 11 = 0 en material público.

---

## 8. VERIFICACIÓN TÉCNICA ADICIONAL

- **Compilación local del core:** `cargo check -p vantadb --all-targets` con la toolchain pineada (1.95.0, definida en `rust-toolchain.toml`) → **PASS en 7m41s, 0 errores, 0 warnings**. Consistente con el gate `clippy -D warnings` del propio CI.
- **Nota de toolchain:** el comentario del propio `rust-toolchain.toml` registra el bug de wasm-pack en Windows con el pin 1.95.0 (STATUS_STACK_BUFFER_OVERRUN) y reconoce que el estable actual (1.98) ya lo contiene — **un bump de canal es deuda pendiente documentada**.
- **Verificación de hallazgos:** C1-C10 (críticos de seguridad/infra) fueron confirmados leyendo el código fuente directamente, no solo por reporte de agentes.
- **Paridad openapi/router:** verificada ejecutando `node scripts/check_openapi_parity.mjs` → 37 paths / 44 operaciones, match exacto.

---

## 9. CONCLUSIÓN FINAL

VantaDB es un proyecto de **ingeniería genuinamente superior al promedio** con una cultura de honestidad que la mayoría de proyectos comerciales no tiene: core de 99K LOC con 7 unwraps y lints anti-unwrap, WAL con auto-reparación testada con caos y Miri, una superficie MCP de 87 tools con meta-tests que asertan la superficie exacta, y una documentación que traza cada decisión a su task ID.

Sus problemas no son de diseño sino de **consolidación y sincronización**: un rename de API que dejó cadáveres (bench script, Colab, hooks), un sistema de gobernanza que ya genera sus propios bugs (CHANGELOG duplicado, trigger corrupto, ADR duplicado, contadores stale), módulos excelentes esperando un host que no existe (scheduler de vanta-memory, mappers SSE del proxy, dashboard), y un embudo público donde las promesas centrales ("1 comando", "privacidad local", "números verificables") fallan hoy en sus tres primeras verificaciones.

**La paradoja central del proyecto:** la capa cuyo único trabajo es que un agente no tenga información stale es la más afectada por información stale (skills, llms.txt, api-reference, benchmarks §2). Y el producto que vende privacidad tiene endpoints sin autenticar que permiten leer y escribir archivos arbitrarios.

**Ruta sugerida:** 2-4 semanas de fixes P0 (casi todos mecánicos) → EXE-03 con 5 testers reales → EXE-02 head-to-head publicado → lanzamiento con la historia "memoria local más profunda para agentes, verificable". La intersección que ofrece es real y nadie más la ofrece; el resto es ejecución.

---

*Informe generado a partir de: clonación de `develop` (2,992 archivos), 12 análisis especializados en paralelo (core, server/bindings, desktop, docs, CI/bench, vanta-memory, proxy/server profundo, MCP/skills, bindings/wasm, providers/examples/benches, src+docs restantes, producto), verificación manual de los 12 hallazgos críticos, y compilación local verificada. Todos los números de LOC son aproximados medidos con `wc -l`.*
