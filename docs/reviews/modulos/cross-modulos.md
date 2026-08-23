# Review Cross-Módulos — VantaDB como Sistema

> **Fecha:** 2026-08-23 · **Revisor:** ox-alpha (segunda opinión, contexto fresco — P2-01)
> **Alcance:** evaluación del SISTEMA COMO UN TODO sobre la base de los 13 reportes individuales (`core.md`, `vantadb-mcp.md`, `vantadb-python.md`, `vantadb-server.md`, `vantadb-ts.md`, `vantadb-wasm.md`, `vantadb-node.md`, `vanta-memory.md`, `vanta-proxy.md`, `providers.md`, `integrations.md`, `benches.md`, `benchmarks.md`) + verificación dirigida propia de los puntos de unión.
> **Método:** lectura de los 13 reportes como mapa; verificación puntual de cada punto de unión crítico contra el fuente (workspace, OpGate ×3, límites por binding, semántica score/distance hasta el core, persistencia WASM, release pipeline, wiring memory→MCP, metrics del proxy). No se re-ejecutaron suites completas.

---

## Dictamen

| Campo | Valor |
|---|---|
| **Veredicto** | 🔴 **Cambios requeridos** (a nivel sistema) |
| **Score conjunto** | **6.5 / 10** |
| **Contrato de integración** | **No pasó** — existen al menos 5 contratos cruzados contradictorios o rotos entre módulos que compilan y testean bien individualmente pero fallan como sistema |

Los módulos individuales promedian **7.05/10** (ver §7). El sistema pierde ~0.5 puntos por defectos que ningún reporte individual podía ver: contratos que divergen entre transportes, duplicación estructural ya derivando, distribución rota fuera de crates.io, y un bug de durabilidad del core expuesto exactamente en el consumidor que más lo sufre.

---

## 1. Mapa de Dependencias Real (verificado)

```
                                 ┌──────────────────────────────────────┐
                                 │   CORE: src/ (crate "vantadb")       │
                                 │   StorageEngine · VantaEmbedded SDK  │
                                 │   error.rs tipado (30 variantes)     │
                                 └───┬────┬────┬────┬────┬────┬───────┘
                                     │    │    │    │    │    │
      workspace members ─────────────┘    │    │    │    └──────────────┐
      (Cargo.toml L622-630)               ▼    ▼    ▼                   ▼
        ┌─────────────────┐  ┌──────────┐ ┌──────────┐ ┌─────────┐ ┌───────────┐
        │ vantadb-python  │  │vantadb-  │ │ vantadb- │ │ vanta-  │ │ vanta-    │
        │ (PyO3, 69% API) │  │mcp       │ │ server   │ │ memory  │ │ proxy     │
        │                 │  │(59 tools,│ │ (thin bin│ │ (L0-L3  │ │ (auth/rate│
        │                 │  │thin      │ │ re-export│ │ pipeline│ │ /inject/  │
        │                 │  │wrapper)  │ │ cli_svr) │ │ )       │ │ /writeback│
        └────────┬────────┘  └──────────┘ └────┬─────┘ └────▲────┘ └────▲──────┘
                 │                             │            │           │
                 ▼                             │            └───────────┘
        ┌─────────────────┐                    │            (proxy → memory)
        │ integrations/   │                    │
        │ 9 adapters Py   │          flujo agente real:
        │ (pip: NO PyPI)  │          OpenCode/Claude → stdio JSON-RPC
        └─────────────────┘          → vantadb-mcp → VantaEmbedded

 FUERA del workspace:
 ├─ providers/{openai,ollama,litellm} → core (path dep, [workspace] propio)
 │    tests rotos · .pyi mienten · sin wheels
 ├─ fuzz/ (nocturno Linux, excluido)
 ├─ JS/NPM (sin release automation):
 │    vantadb-wasm ──pkg/ file-dep──► vantadb-ts (wrapper)
 │    vantadb-node (napi) ◄──NativeVantaDB (subset)
 └─ desktop/ (Tauri) → ServerClient HTTP → cli_server
      o embebido WASM (OPFS). HEREDA: grafo no persiste (CORE-02),
      fallback OPFS silencioso, y H-1 si usa backend InMemory.
```

**Hechos verificados del workspace:**

- Members: `.`, `vantadb-python`, `vantadb-server`, `vantadb-mcp`, `vantadb-wasm`, `vanta-memory`, `vanta-proxy` (`Cargo.toml:622-630`). `default-members` reducido a `.` + `vantadb-python` (L637-640); server/mcp/wasm marcados EXPERIMENTAL fuera del gate CI principal (L641-643).
- Exclusiones correctas y documentadas: `fuzz/` (L633), `providers/*` por crash MSVC linker (nota L634-636 — CRIT-09 cerrado coherentemente según reporte providers).
- `desktop/` existe físicamente pero está **fuera del workspace** (desacoplado ✅); se conecta vía `ServerClient` HTTP (`desktop/src-tauri/src/connections/server.rs`) o WASM embebido.
- Versión única `0.5.0` vía `[workspace.package]` (L646): los 7 members Rust sincronizados por construcción ✅. Los package.json de ts/node (ambos 0.5.0) e integrations (9× 0.5.0) se sincronizan **a mano** — frágil.

### Flujo de release real (release-plz.toml + release.yml)

| Canal | ¿Cubierto? | Evidencia |
|---|---|---|
| Crates.io: workspace crates | ✅ | `release.yml` → release-plz Trusted Publishing |
| `vantadb-wasm` (npm vía wasm-pack) | ❌ | `release-plz.toml:28-32` `release = false`; publicación npm manual o inexistente |
| `vantadb-ts` / `vantadb-node` (npm) | ❌ | 0 menciones npm en `release.yml`; node sin optionalDependencies multiplataforma (reporte node #2) |
| PyPI: `vantadb-py` + 9 integrations + 3 providers | ❌ | 0 menciones pypi/maturin/twine en `release.yml`; MKT-18f vigente (404 verificado en reportes) |
| Gates de regresión perf | ❌ muertos | `criterion_baseline.json` y `python_baseline.json` **vacíos** (reporte benchmarks 🔴) |

**Conclusión:** el único canal automatizado es crates.io. Todo el ecosistema de consumo real de una DB local-first para agentes (npm + PyPI) vive fuera del release pipeline: la adopción exige install-from-git o compilación manual.

---

## 2. Matriz Módulo×Módulo de Contratos Verificados

Leyenda: ✅ verificado sano · ⚠️ funciona con divergencia documentada · ❌ contrato roto o contradicción · ➖ sin relación directa

| Consumidor ↓ / Proveedor → | core (SDK/engine) | vantadb-python | vantadb-wasm (+pkg) | vantadb-node | vantadb-mcp | cli_server (HTTP) | vanta-memory | docs/skills públicos |
|---|---|---|---|---|---|---|---|---|
| **vantadb-mcp** | ✅ thin-wrapper ejemplar; serde verbatim; test paridad nativo↔MCP | ➖ | ➖ | ➖ | — | ⚠️ dos servidores paralelos (stdio vs axum), semántica coherente | ⚠️ skills/wiki delegan OK; scene handlers SIN wiring | ✅ SKILL.md hash-SAME; 59 tools contadas vs source |
| **integrations/** (9 adapters) | vía vantadb-py | ✅ imports/firmas verificados correctos | ➖ | ➖ | ➖ | ➖ | ➖ | ⚠️ metadata publicable engañosa (no hay PyPI) |
| **vantadb-ts** | — | ➖ | ❌ tipos de grafo ficticios vs `Vec<u128>` (#1); ❌ `_native` sin await-catch (#2) | ⚠️ subset ~11 métodos hereda límites | ➖ | ➖ | ➖ | ❌ JSDoc `distance` invertido (#3) |
| **vantadb-node** | ⚠️ ~11 ops de ~42 (5× menos que wasm) | ➖ | ➖ | — | ➖ | ➖ | ➖ | ❌ sin README; index.d.ts todo `any`; distribución rota fuera win-x64 |
| **vanta-memory** | ✅ consume SDK público; degradación P4 sistemática | ➖ | ➖ | ➖ | ❌ handlers scene_* listos, 0 wiring MCP | ⚠️ solo indirectamente vía proxy | — | ⚠️ F7 narra "12 tools MCP" inexistentes |
| **vanta-proxy** | ⚠️ default-features=false (grafo acíclico ✅) | ➖ | ➖ | ➖ | ➖ | ➖ | ✅ inyección/captura/writeback verificados e2e | ❌ C-1: `mem:` commands responden éxito simulado |
| **desktop/** Studio | vía HTTP/WASM | ➖ | ❌ hereda: grafo no persiste (CORE-02) + fallback OPFS silencioso | n/a fuera Windows | ➖ | ❌ búsqueda textual presumiblemente rota en DB fresca (server 8.1) | ➖ | ➖ |
| **providers/** ×3 | ✅ path-dep correcto, GIL OK | ❌ tests llaman APIs inexistentes (`create_namespace`) | ➖ | ➖ | ➖ | ➖ | ➖ | ❌ .pyi mienten (P3); READMEs desactualizados |
| **benches/benchmarks** | ⚠️ miden features vendidas con huecos (WAL, grafo) | ✅ BENCH-01 usa SDK real | ⚠️ wasm_bench sin output commiteado | ➖ | ➖ | ➖ | ➖ | ⚠️ claims honestos pero WASM report sin datos crudos |

---

## 3. Patrones Cross-Cutting (con evidencia propia)

### CC-1 — OpGate triplicado verbatim (confirmado)

`struct OpGate` existe en tres copias: `vantadb-wasm/src/lib.rs:306`, `vantadb-python/src/lib.rs:91`, `vantadb-node/src/lib.rs:201`. Misma barrera, mismos comentarios. **Riesgo sistémico:** el fix del deadlock potencial de Python (H2 python: `drain()` sosteniendo el GIL) no se propaga automáticamente a las otras dos copias si alguien lo corrige en una sola. Extraer a crate compartido (`vantadb-gate` o módulo del core feature-gated).

### CC-2 — Errores tipados de élite destruidos en cada frontera FFI

El core tiene la mejor taxonomía de errores del proyecto (30 variantes, source-chaining, hints, retry-classification — fortaleza #1 del reporte core). Cada binding la aplana:

- wasm: `to_js_err` → string plano (`lib.rs:1518`)
- node: `napi::Error::from_reason(Display)` — string plano
- python: todo → builtins genéricos, catch-all `RuntimeError` (`convert.rs:659-684`)
- TS intenta reconstruir códigos propios (`WASM_ERROR`) sobre texto ya perdido

**Consecuencia sistémica:** ningún consumidor puede manejar programáticamente `NotFound` vs `Timeout` vs `InvalidInput` de forma uniforme salvo parseando strings. El activo de ingeniería más valioso del core muere en la frontera.

### CC-3 — Declaraciones desincronizadas ×2

- `vantadb-python`: dos `.pyi` casi idénticos derivando por separado (H3).
- `providers/` ×3: `.pyi` firman una API que ya no existe (P3).

Causa raíz común: declaraciones manuales junto a código que evoluciona. El fix correcto es un patrón único (test de drift firmas↔stub), no tres patches.

### CC-4 — Contratos ficticios / éxito simulado (≥6 instancias)

| Instancia | Módulo | Evidencia |
|---|---|---|
| `GraphBfsResult/DfsResult/TopoSort` ≠ wire real (`Vec<u128>`) | ts #1 | blind-cast `as GraphBfsResult`; tests afirman `toBeDefined()` |
| `mem:sync`/`mem:create-skill` responden "✅" sin efecto | proxy C-1 | `mem_command.rs:104-107` |
| `trigger_compaction()` loguea acción inexistente | core M-1 | `maintenance.rs:22-48` |
| Tests de providers llaman APIs inexistentes | providers P1/P2 | suites rojas (`create_namespace`, firma vieja de search) |
| Brazo criterion falso (cronometra un entero) | benches 🔴 | `memory_budget` `rss_vs_dataset_trend` |
| Test rate-limit acepta `200||429` | server 8.3 | valida "responde", no el límite |

Patrón de fondo: **la superficie declara más de lo que entrega**, y los tests prueban presencia, no forma.

### CC-5 — Los mismos 5 gaps de API faltan en TODOS los transportes

Comparando las secciones de cobertura de los 5 reportes de bindings: `remove_edge`, `versions/get_version/supersede`, `similar_to_key`, `count`, `delete_by_filter` están ausentes simultáneamente en MCP (§5), Python (H9), WASM (§5), TS (§5) y node (§5). No son bugs independientes: **es un gap de decisión de producto** — nadie definió la superficie mínima común de los transportes. Consecuencias encadenadas: mem0 vive de fallbacks por métodos inexistentes (I7), edges son inborrables desde cualquier lenguaje, historial de versiones inalcanzable para agentes.

### CC-6 — Límites divergentes por transporte (verificado hoy)

```
                vector dim máx      top_k máx
wasm            10_000_000          1_000     (MAX_F32_VEC_LEN/MAX_K, wasm lib.rs:38,43)
python          validación core     1_000     (MAX_K, python lib.rs:43)
node            10_000              10_000    (MAX_VEC_DIM, node lib.rs:25)
```

Misma operación, contratos 1000× distintos según transporte. Además `distance_metric` case-sensitive en wasm pero tolerante en node (node #6). Las constantes deberían vivir en el core.

### CC-7 — Semántica score-vs-distance resuelta hasta el fuente (nuevo hallazgo de conexión)

Verifiqué la cadena completa:

- **Core:** `score` = similitud. Evidencia: `similar_to_key` computa `score: 1.0 - hit.distance` (`src/sdk/api.rs:1555`). `search_vector` devuelve `{node_id, distance}` crudo (`api.rs:1320`).
- **WASM:** dos sites con el mismo nombre de campo y semántica opuesta **dentro del mismo binding**: `search_hit_to_js` expone `"score" = hit.score` (similitud, `lib.rs:953`); `search_vector` expone `"score" = hit.distance` (distancia, `lib.rs:1024`). El binding se contradice a sí mismo.
- **TS:** mapea ese campo a `.distance` documentado "lower = more similar" (`vantadb.ts:579`) — correcto para search_vector, **invertido** para search de memoria.
- **Node:** test afirma similitud (`persistence.test.ts:96-98`).

Un usuario que ordena hits según el JSDoc de TS obtiene resultados invertidos en la ruta de búsqueda de memoria. La decisión debe tomarse UNA vez en el core (exponer ambos campos `score`+`distance` o elegir uno y documentarlo) y propagarse.

### CC-8 — Infraestructura construida y nunca cableada

Repetido en 4 módulos: mecanismos completos y testeados sin caller en producción.

- vanta-memory: handlers `scene_read/list/query` implementados, 0 wiring MCP/REST (grep propio confirmado: 0 refs en `vantadb-mcp/src` ni `vanta-proxy/src`)
- proxy: session state machine `advance()` sin caller (~330 líneas dormidas); `Reporter.add_hook` sin hooks registrados
- server: feature `sysinfo = []` vacía sin consumidor
- benchmarks: dos workflows de regresión apuntando a baselines vacíos
- core: PITR funcional pero desconectado (ADR-014)

Deuda de integración, no de calidad de código: cada pieza funciona sola; ninguna está conectada.

---

## 4. Errores de Conexión / Flujo entre Módulos (tabla de severidad)

| # | Sev | Flujo roto | Evidencia | Impacto en el sistema |
|---|---|---|---|---|
| F-1 | 🔴 | **core H-1 (WAL resurrect) expuesto vía WASM.** El reporte wasm documenta que `build_config` fuerza `BackendKind::InMemory` (`wasm lib.rs:72`) — exactamente el motor legacy donde vive H-1 (insert/update escriben WAL antes de validar; replay resurrecta writes rechazados, `engine.rs:228/150-166`). Todo navegador que use OPFS/IDB hereda un bug de corrupción de datos alcanzable por uso normal. | core.md H-1 + wasm.md §2 | Corrupción silenciosa post-reload para todo consumidor browser (incluido Studio en modo embebido). El fix del core (R1) debe priorizarse no por el core sino por este consumidor. |
| F-2 | 🔴 | **HTTP: búsqueda textual/híbrida presumiblemente rota en DB fresca** (`ensure_indexes_current` no corre en `cli_server::run`; solo `VantaEmbedded::open_with_config` lo hace). Studio desktop vía bridge HTTP y cualquier cliente REST obtienen "text_index not found: bm25". | server 8.1 con cadena completa `cli_server.rs:1758→builder.rs:35-42` | Contrato REST de búsqueda roto para DB fresca; los tests e2e no lo ven porque solo usan IQL INSERT/FETCH/DELETE. |
| F-3 | 🔴 | **MCP: notifications JSON-RPC rechazadas** (`RpcRequest.id` no-Option → `-32700` espuria ante `notifications/initialized`). Todo cliente MCP estricto falla el handshake o trata la línea espuria como error de sesión. | mcp H1 (`protocol.rs:8-14`, `server.rs:102`) | Puerta de entrada principal del flujo agente real. Fix ~10 líneas. |
| F-4 | 🔴 | **WASM: grafo no persiste** — `save()/save_idb()` serializan solo records (`persist_payload`, lib.rs:661-720); verificado hoy que `insert_node` (L1257) y `add_edge` (L1307) nunca tocan `PersistCache`. Cross-session, nodos y edges desaparecen. | wasm #1 (hipótesis fuerte para CORE-02) | Studio/local-first browser pierde todo GraphRAG al recargar. |
| F-5 | 🔴 | **Python: gate por defecto roto** — `pytest -q` = 66 failed / 43 passed por RSS acumulado sin teardown; claim "70 passed" era solo `test_sdk.py`. | python H1 (run completo documentado) | CI del binding principal no puede ser verde; el contrato del módulo no pasa. |
| F-6 | 🟠 | **Score/distance contradictorio entre transportes** (CC-7): wasm se contradice a sí mismo; TS JSDoc invertido en la ruta search. | `wasm lib.rs:953 vs 1024`; `api.rs:1555` | Ordenamiento incorrecto de hits por usuarios que confían en docs. |
| F-7 | 🟠 | **node indistribuible fuera de Windows**: binario único commiteado, sin optionalDependencies; quien instala en Linux/macOS falla en runtime. `NativeVantaDB` de TS hereda la rotura. | node #2 | Backend nativo prometido ("todo lo que hace WASM + persistencia real") inutilizable fuera de win-x64. |
| F-8 | 🟠 | **vanta-memory sin transporte** pese a handlers completos; el flujo proxy→write-back→memory→core funciona pero la capa knowledge (scenes/wiki query) es inalcanzable desde agentes. | memory N-2 + grep propio | La narrativa F7 ("12 tools MCP") es ficticia; Studio no puede consumir scenes salvo embebido Rust. |
| F-9 | 🟡 | **proxy sin metrics endpoint** — DESKTOP-38 (observabilidad del proxy para Studio) bloqueado: solo JSON log lines, hooks no conectados, cero Prometheus/metrics route (grep propio: 0 hits en `vanta-proxy/src/server.rs`). | proxy §7 (DEFER documentado) + verificación | Studio no tiene señal de salud del proxy salvo parsear logs. |
| F-10 | 🟡 | **providers tierra de nadie**: tests rojos contra API actual, `.pyi` mintiendo, sin pyproject/wheels, sin CI que los compile (fuera del workspace). | providers P1-P4 | Tres crates "públicos" sin camino de instalación ni señal de calidad. |
| F-11 | 🟡 | **integrations no publicadas** (MKT-18f): 9 adapters con metadata seria, 0 en PyPI. El paso 1 del flujo de uso real (`pip install vantadb-langchain`) falla. | integrations I1 (404 verificado) | Bloqueador directo de adopción del ecosistema Python. |
| F-12 | 🟡 | **Gates de perf muertos**: ambos baselines vacíos; workflows comparan contra nada. El trabajo pesado existe; falta una corrida bootstrap. | benchmarks 🔴 ×2 | No-regresión declarada = no-op hoy. |
| F-13 | 🟢 | **pkg/ file-dependency** (`vantadb-ts` ← `vantadb-wasm/pkg`): artefacto de build commiteado; drift posible si se regenera con otra versión. | ts #9 | Riesgo de desincronización silenciosa wasm↔ts. |

---

## 5. Deficiencias de Calidad Conjunta

1. **Sin contrato de superficie común.** Cada transporte define su propio subconjunto de la API core (mcp 90%, python 69%, wasm ~45 ops, ts ~35, node ~11) sin una matriz canónica de "qué debe exponer todo transporte". CC-5 y CC-6 son síntomas.
2. **Duplicación estructural ya derivando.** OpGate ×3 (CC-1), `_mapRecord/_buildSearchRequest` duplicados ts↔native (ts #5), opfs.rs↔opfs_bridge.js (wasm #14), providers 85% copia-pega con API inconsistente resultante (P5), ollama/openai integrations gemelos (I8), dos `.pyi` (H3). En cada caso la divergencia ya produjo un bug real.
3. **Versionado/release asimétrico.** Rust workspace sincronizado ✅, pero npm/PyPI fuera de automatización (§1). Los package.json sincronizados a mano son una cuenta regresiva.
4. **Tests prueban presencia, no contratos** (CC-4): `toBeDefined()`, `200||429`, brazo criterion falso, suites que llaman APIs inexistentes. El sistema tiene muchos tests pero pocos *contratos* testeados en las fronteras.
5. **Docs públicas por delante del código.** Skills narran tools MCP inexistentes (F7 scenes), JSDoc miente sobre semántica (distance), stubs mienten sobre firmas (×2), READMEs desactualizados (providers P8). Para un producto agentic donde la doc ES la interfaz del LLM, esto es más grave que en software convencional: un agente que confía en SKILL.md/JSDoc produce llamadas rotas.
6. **Cultura de honestidad excelente en benches, ausente en superficie.** El eje PERF-03/MKT-18g publicó derrotas tal cual (ejemplar). Pero mem_command "✅" simulado, trigger_compaction logueando acción inexistente y tipos TS ficticios son lo opuesto en la capa funcional.

---

## 6. Riesgos Sistémicos Priorizados

| Rank | Riesgo | Por qué sistémico | Mitigación |
|---|---|---|---|
| 1 | **Corrupción silenciosa (H-1 + F-4)** en todo consumidor browser/WASM y en el path legacy del core | Datos del usuario alterados tras restart/reload sin error visible — el peor modo de fallo para un motor de memoria persistente | Core R1 (validate→WAL→apply) + migrar wasm fuera de InMemoryEngine + persistir grafo |
| 2 | **Contratos contradictorios entre transportes** (score/distance, límites, errores-as-string) | Cada fix local aumenta divergencia; usuarios multi-transporte (TS wrapper sobre node nativo, p.ej.) reciben semánticas distintas según backend elegido | Decisión única en core + propagación (ver Propuesta A) |
| 3 | **Handshake MCP roto (F-3)** | Es LA puerta de entrada del producto para agentes; un cliente estricto nuevo puede fallar desde el minuto cero | Fix de 10 líneas + test de notifications |
| 4 | **Distribución inexistente fuera de crates.io** | Producto local-first cuyo valor depende de instalarse fácil en Node/Python/browsers; hoy requiere git+build manual | Pipeline npm/PyPI (Propuesta D) |
| 5 | **Deuda de integración acumulándose** (CC-8): piezas listas sin cablear | Cada pieza dormida es costo de mantenimiento sin retorno y falsa sensación de feature-complete en planning | Wiring explícito o delete consciente |

---

## 7. Score Conjunto

### Scores individuales (de los 13 reportes)

| Módulo | Score | Veredicto |
|---|---|---|
| vanta-memory | 8.5 | ✅ mejor crate del proyecto |
| core (src/) | 8.3 | ✅ con seguimiento (H-1/H-2) |
| vantadb-mcp | 8.3 | ✅ con seguimiento (H1/H3 protocolo) |
| vanta-proxy | 8.0 | ✅ con C-1 de honestidad funcional |
| vantadb-server | 7.5 | 🔴 8.1 bloquea contrato REST de búsqueda |
| vantadb-wasm | 7.0 | ⚠️ grafo no persiste (CORE-02) |
| vantadb-ts | 7.0 | ⚠️ tipos grafo ficticios, _native bug |
| benches | 7.0 | ⚠️ brazo falso + huecos WAL/grafo |
| benchmarks | 6.5 | ⚠️ gates muertos, 128 MB commiteados |
| integrations | 6.5 | ⚠️ MKT-18f + 4 bugs latentes |
| vantadb-python | 6.5 | 🔴 suite default rota + deadlock potencial |
| providers | 5.0 | 🔴 tests rotos, .pyi mienten, sin distribución |
| vantadb-node | 4.5 | 🔴 11/42 ops, distribución rota |
| **Media aritmética** | **7.05** | |

### Ajuste sistémico

| Factor | Delta |
|---|---|
| Arquitectura de dependencias limpia (grafo acíclico, thin wrappers, canonicalidad verificada server/mcp) | + |
| Versión workspace única y exclusiones documentadas | + |
| Contratos cruzados contradichos (score/distance, límites, tipos ficticios) | − |
| Duplicación estructural ya generando bugs (OpGate, .pyi ×2, providers) | − |
| Distribución rota/inexistente en 3 de 4 canales | − |
| Infraestructura lista sin cablear (memory→MCP, baselines, state machine) | − |
| Bug de durabilidad del core expuesto en el consumidor más vulnerable (WASM) | − |

### **Score conjunto del sistema: 6.5 / 10**

Un proyecto cuya calidad *interna* por módulo es notablemente alta (varios módulos 8+) pero cuya *integración* está a medio hacer: los contratos entre piezas no fueron diseñados como sistema, sino que emergieron por duplicación y luego divergieron. El gap entre 7.05 (módulos) y 6.5 (sistema) mide exactamente eso.

---

## 8. Propuestas de Arquitectura

### A. Contrato único en el core (máximo leverage)
Definir EN el core y exportar: (1) constantes de límites (`MAX_K`, vector dim) importadas por todos los bindings; (2) semántica de score/distance — recomendación: exponer AMBOS campos en cada hit (`score`=similitud normalizada, `distance`=crudo) con doc única; (3) taxonomía de errores serializable `{code, message, hint}` que cada frontera FFI traduzca mecánicamente (enum discriminado → código string estable). Elimina CC-2/CC-6/CC-7 por construcción.

### B. Crate compartido para bindings
Extraer `OpGate` + helpers comunes a un crate interno (feature-gated o `path-dep`). Tres copias → una. El próximo fix de concurrencia se propaga gratis. Idéntico patrón para `providers/common` (P6, ~500 líneas).

### C. Matriz de superficie mínima común
Documento de una página: qué métodos debe exponer TODO transporte (propuesta inicial: los de CC-5 — put/get/delete/list/search/search_vector/count/delete_by_filter/remove_edge/versions/supersede/similar_to_key) + qué es opt-in. Test cross-crate que falle si un transporte declara paridad y le falta un método del conjunto mínimo. Esto convierte los gaps accidentales en decisiones explícitas.

### D. Pipeline de distribución único
Extender release.yml: job npm (wasm-pack build → publish wasm + ts + node con optionalDependencies por plataforma) y job PyPI (maturin para vantadb-py + hatchling para 9 integrations). Sin esto, toda la campaña de reviews optimiza un producto que nadie puede instalar fácil.

### E. Consolidación ponytail (deuda de CC-8)
Decidir explícitamente por cada pieza dormida: cablear o borrar. Concretamente: wiring MCP de scene handlers (~día, desbloquea la narrativa F7 completa) O marcar DEFER formal en backlog; session state machine del proxy O se consume O se congela tras un doc-comment; feature sysinfo vacía → delete.

---

## 9. Top-5 Acciones (priorizadas y verificables)

1. **Fix durabilidad cross-módulo:** core H-1 (reordenar validate→WAL→apply + test de resurrection) Y wasm persistencia de grafo (extender `db_state.json` a `{records, nodes, edges}` o snapshots del core). Cierra F-1/F-4/CORE-02. Verificación: test insert-duplicado-rechazado→reopen→payload intacto; test wasm save→reload→edges presentes.
2. **Fixes de puertas de entrada (un día total):** notifications JSON-RPC en mcp (H1+H3, ~13 líneas + test); `ensure_indexes_current` en `cli_server::run()` (server 8.1, 1-3 líneas + e2e text-search); suite pytest verde (teardown fixture, python H1).
3. **Decisión de contrato único en core (Propuesta A):** score/distance + límites + shape de error. Verificación: un test de paridad por transporte afirmando el mismo valor para el mismo input.
4. **Activar distribución:** bootstrap de los 2 gates de bench (corrida + commit de baseline) y primer pipeline npm/PyPI (aunque sea manual-documentado al principio). Verificación: `pip install vantadb-py==X` y `npm install vantadb-ts@X` funcionan desde registry.
5. **Wiring vanta-memory→MCP:** exponer scene_read/list/query como tools en vantadb-mcp (los handlers existen y están testeado; falta el dispatch). Desbloquea F8, cumple la narrativa F7 y da a los agentes acceso a la capa knowledge.

---

## 10. Notas de Verificación de Este Reporte

- Todos los hallazgos propios citan `file:línea` verificados por lectura directa durante esta sesión (Cargo.toml, release-plz.toml, release.yml, wasm lib.rs ×5 sites, node lib.rs, python lib.rs, api.rs via codegraph).
- Los hallazgos heredados de los 13 reportes se citan por su ID original (ej. "server 8.1", "python H1"); su evidencia vive en el reporte correspondiente y no fue re-verificada línea a línea salvo los puntos de unión críticos marcados "(confirmado)" o "(verificado hoy)".
- No se ejecutaron suites de tests (alcance de segunda opinión: verificación puntual, no re-ejecución).

---

*Ver reportes individuales: [`core.md`](./core.md) · [`vantadb-mcp.md`](./vantadb-mcp.md) · [`vantadb-python.md`](./vantadb-python.md) · [`vantadb-server.md`](./vantadb-server.md) · [`vantadb-ts.md`](./vantadb-ts.md) · [`vantadb-wasm.md`](./vantadb-wasm.md) · [`vantadb-node.md`](./vantadb-node.md) · [`vanta-memory.md`](./vanta-memory.md) · [`vanta-proxy.md`](./vanta-proxy.md) · [`providers.md`](./providers.md) · [`integrations.md`](./integrations.md) · [`benches.md`](./benches.md) · [`benchmarks.md`](./benchmarks.md)*
