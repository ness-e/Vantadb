---
name: review-deep
description: >
  Deep review & optimization loop. Iterates through every module of VantaDB
  (Rust core, Python SDK, TS SDK, adapters) excluding web/. For each module:
  deep analysis → web research → competitor compare → triage → backlog.
  Uses ALL available skills, tools, plugins. Loop-driven via /loop-goal.
compatibility: opencode
---

# Review Deep — Loop de Revisión Profunda

> **Diferencia con vantadb-full-review:** no es un reporte one-shot.
> Es un **loop que itera módulo por módulo**, investiga cada hallazgo en internet,
> compara con competidores, evalúa prioridad real, y lo agrega a Backlog.md.
> Corre tantas iteraciones como módulos tenga el proyecto.

---

## Arquitectura del Loop

```
/loop-goal --prompt-file .opencode/skills/review-deep/loop-prompt.md "MODULE=vantadb::sdk" "DEPTH=full"
  │
  ├─ FASE 0: Cargar skills según tipo de módulo
  │
  ├─ FASE 0b: Tool lock-in por fase  ← statewright: ~5 tools visibles, no 30+
  │   ├─ F1-F3 (análisis): solo read + codegraph + search
  │   ├─ F4-F5 (research): solo search + browser
  │   ├─ F6 (triage): solo edit (Backlog.md) + write
  │   └─ Transiciones explícitas: no saltar de F3 a F6 sin pasar por F4-F5
  │
  ├─ FASE 1: CodeGraph structural mapping
  │   ├─ codegraph_explore "module symbols dependencies"
  │   ├─ codegraph_explore "module callers callees"
  │   └─ Mapear: callers, callees, API surface, tests
  │
  ├─ FASE 2: Static analysis (tools mecánicos)
  │   ├─ Rust: cargo check, clippy -D, machete, outdated, audit, deny
  │   ├─ Python: mypy, pytest, ruff
  │   ├─ TS: tsc --noEmit, eslint, vitest
  │   └─ Registrar: warnings, errors, issues
  │
  ├─ 🟢 QUALITY GATE 1 (entre F3→F4)
  │   ├─ ¿Todos los hallazgos tienen archivo:línea?
  │   ├─ ¿Cada hallazgo tiene tipo + severidad?
  │   ├─ Si faltan → volver a F3 antes de seguir
  │   └─ quality in the loop: no esperar al final para validar
  │
  ├─ FASE 3: Deep code review (humano-asistido con codegraph)
  │   ├─ unsafe blocks → SAFETY docs presentes?
  │   ├─ expect/unwrap → justificados o reemplazables?
  │   ├─ Error handling → errores ignorados? panic en library?
  │   ├─ Performance → clones, allocs, O(n²), lock contention?
  │   ├─ Concurrency → race conditions, deadlocks, ordering?
  │   ├─ Security → input validation, path traversal, crypto?
  │   ├─ Architecture → god modules, circular deps, coupling?
  │   ├─ Clarity → naming, magic numbers, dead code?
  │   └─ Testing → coverage, edge cases, flaky?
  │
  ├─ FASE 4: Web research por cada hallazgo
  │   ├─ MetaSearchMCP.search_web("patrón/issue específico")
  │   ├─ Argus.extract_content("url con solución/documentación")
  │   ├─ ¿Hay librería mejor? ¿patrón más moderno?
  │   └─ Registrar: source URL, solución recomendada, alternativas
  │
  ├─ FASE 5: Competitor comparison
  │   ├─ Chroma / Pinecone / Qdrant / Milvus / LanceDB
  │   ├─ ¿Tienen esta feature? ¿La nuestra es mejor/peor?
  │   ├─ ¿Benchmarks públicos? ¿Cómo nos comparamos?
  │   └─ Registrar: gap/ventaja competitiva
  │
  ├─ 🟢 QUALITY GATE 2 (entre F5→F6)
  │   ├─ ¿Cada hallazgo Medium+ tiene research asociada?
  │   ├─ ¿Las URLs de referencia son válidas?
  │   ├─ Si falta investigación → volver a F4
  │   └─ Calidad sobre cantidad: descartar falsos positivos
  │
  ├─ FASE 6: Triage → Backlog.md
  │   ├─ Evaluar: severidad, impacto, esfuerzo, prioridad real
  │   ├─ ¿Es rápido de arreglar? → hacerlo ahora (🟢 <30min)
  │   ├─ ¿Requiere plan? → agregar a Backlog.md con ID único
  │   ├─ ¿No es relevante? → descartar con razón
  │   └─ Actualizar Backlog.md + progreso
  │
  └─ FASE 7: Reporte del módulo + yield
      ├─ Resumen: hallazgos N, críticos M, mejoras K
      ├─ Si hay más módulos → yield para que el loop lo invoque de nuevo
      └─ Si es el último → reporte consolidado + complete
```

---

## Orden de Iteración de Módulos

El loop itera en este orden, de mayor a menor impacto potencial:

```
Wave 0 — Core crítico (3 módulos):
  vantadb-sdk       → VantaEmbedded, connect(), Vanta* types
  vantadb-engine    → engine.rs, storage backends
  vantadb-wal       → Write-Ahead Log, recovery

Wave 1 — Indexación y vectores (2 módulos):
  vantadb-vector    → HNSW, distance, quantization, governor
  vantadb-index     → flat, graph, core

Wave 2 — Gobernanza (1 módulo):
  vantadb-governance → admission, consistency, conflict

Wave 3 — SDKs y bindings (3 módulos):
  vantadb-python    → PyO3 bindings
  vantadb-ts        → TypeScript SDK
  vantadb-wasm      → WASM build

Wave 4 — Infraestructura (2 módulos):
  vantadb-server    → HTTP server, CLI server
  vantadb-mcp       → MCP integration

Wave 5 — Adaptadores (10 módulos, paralelizable):
  vantadb-openai, vantadb-ollama, vantadb-litellm
  vantadb-mem0, vantadb-letta, vantadb-crewai
  vantadb-dspy, vantadb-haystack, vantadb-langchain, vantadb-llamaindex

Wave 6 — Utilidades y misc (3 módulos):
  vantadb-crypto    → encryption/decryption
  vantadb-cli       → CLI handlers
  vantadb-enterprise → enterprise crate
```
<!-- ponytail: waves secuenciales. Si un wave tarda y el siguiente no depende, se podría solapar. DAG solver si >30 módulos. -->

Cada wave espera a que la anterior termine (dependencias naturales).
Dentro de una wave, los módulos se pueden revisar en paralelo.

### Context Preservation (módulos grandes)

Para módulos >2000L, el loop se vuelve caro en tokens (67.6% son tool outputs
— source: awesome-harness-engineering). Estrategia:

| Tamaño | Estrategia |
|--------|-----------|
| <500L | Loop normal, 1 iteración |
| 500-2000L | Compactar hallazgos intermedios en un archivo temporal |
| >2000L | Partir en sub-módulos, invocar sub-loop por cada uno |
| >5000L | Usar sub-agent por sub-módulo en paralelo |

El archivo temporal `.opencode/skills/review-deep/tmp/${MODULE}-findings.json`
guarda hallazgos intermedios para no perder contexto entre iteraciones.

### Scorecard por Iteración (darwin-godel pattern)

Cada iteración del loop registra una entrada en el scorecard:

```
.iterations/${MODULE}-${TIMESTAMP}.json
{
  "module": "vantadb-sdk",
  "wave": 0,
  "duration": "12m34s",
  "findings": { "critical": 2, "high": 5, "medium": 3, "low": 8, "info": 12 },
  "fixed_now": 3,
  "to_backlog": 4,
  "discarded": 3,
  "research_urls": 7,
  "competitor_gaps": 2,
  "previous_comparison": { "prev_total": 18, "delta": -2 }
}
```

Esto permite comparar progreso entre revisiones del mismo módulo y detectar
regresión (nuevos hallazgos vs previos resueltos).

---

## FASE 0: Cargar Skills

| Tipo de módulo | Skills a cargar |
|----------------|----------------|
| Rust core | `ponytail-audit`, `code-review-and-quality`, `doubt-driven-development`, `code-simplification`, `security-and-hardening`, `performance-optimization`, `api-and-interface-design`, `source-driven-development` |
| Python SDK | `code-review-and-quality`, `security-and-hardening`, `source-driven-development` |
| TypeScript SDK | `code-review-and-quality`, `security-and-hardening` |
| Adaptadores | `code-review-and-quality`, `doubt-driven-development`, `ponytail` |
| WASM | `performance-optimization`, `code-review-and-quality` |
| Cualquiera | `skill progreso`, `skill brainstorming` (si el hallazgo requiere diseño), `skill writing-plans` (si requiere plan), `spec-driven-development` (si es feature nueva) |

**Siempre:** `skill progreso` al inicio y al completar cada módulo.
**Siempre:** activar `ponytail full`.

---

## FASE 1: CodeGraph Structural Mapping

Por cada módulo:

```
codegraph_explore "vantadb::${MODULE} symbols classes functions"
codegraph_explore "vantadb::${MODULE} callers"
codegraph_explore "vantadb::${MODULE} callees dependencies"
```

Registrar en la bitácora del módulo:

- **API Surface:** funciones/structs pub exportadas
- **CALLERS:** qué módulos usan este módulo
- **CALLEES:** de qué depende este módulo
- **TEST COVERAGE:** qué tests cubren este módulo
- **FILE SIZES:** archivos grandes (>500L) que son candidatos a split
- **COMPLEXITY:** módulos con太多 responsabilidades

---

## FASE 2: Static Analysis

### Para código Rust

```bash
cargo check -p ${CRATE} 2>&1
cargo clippy -p ${CRATE} --all-targets --all-features -- -D warnings 2>&1
cargo fmt --check -p ${CRATE} 2>&1
cargo machete -p ${CRATE} 2>&1
cargo outdated -p ${CRATE} --exit-code 1 2>&1
cargo audit 2>&1
cargo deny check 2>&1
```

### Para Python SDK

```bash
dev-tools/setup_venv.ps1 2>&1 | tail -5
target/audit-venv/Scripts/python -m pytest vantadb-python/tests/ -v 2>&1 | tail -30
target/audit-venv/Scripts/python -m mypy vantadb-python/ 2>&1 | tail -20
target/audit-venv/Scripts/python -m ruff check vantadb-python/ 2>&1 | tail -20
```

### Para TS SDK

```bash
cd vantadb-ts/
npx tsc --noEmit 2>&1
npx eslint . --ext .ts 2>&1
npx vitest run 2>&1
```

### Para adaptadores

```bash
cargo check -p ${ADAPTER_CRATE} 2>&1
```

---

## FASE 3: Deep Code Review

### 3a. Pattern Scanning (con grep/codegraph)

```
Buscar en ${MODULE}:
  expect(         → count, justify each
  unwrap(         → count, justify each
  unsafe {        → count, verify SAFETY docs
  todo!(          → count, pending implementations
  unimplemented!( → count, missing implementations
  unreachable!(   → count, verify infallibility
  #[allow(        → count, verify justification
  as               → count casts, verify safety
  transmute       → count, verify safety
  clone()         → count, verify need vs Copy types
  .lock()         → count, verify ordering (no deadlock)
  .block()        → count, verify async context
  catch {}        → count, verify empty error swallowing
  let _ =         → count, verify ignored results
```

### 3b. Error Handling Review

- [ ] ¿Todos los `Result` se manejan? (no `ok()` silencioso)
- [ ] ¿Los `expect()` tienen mensaje descriptivo con precondición?
- [ ] ¿Los `unwrap()` son provably infallible?
- [ ] ¿Hay errores que se tragan con `let _ =`?
- [ ] ¿Los errores se propagan con `?` o se envuelven?
- [ ] ¿Los mensajes de error son accionables?

### 3c. Performance Review

- [ ] Hot paths: ¿allocaciones evitables?
- [ ] Hot paths: ¿lock contention? (Mutex/RwLock global vs granular)
- [ ] Algoritmos: ¿O(n²) donde O(n log n) es posible?
- [ ] Serialización: ¿formato eficiente? (bincode vs json)
- [ ] Vector: ¿batch operations o one-by-one?
- [ ] I/O: ¿fsync policy razonable?

### 3d. Concurrency Review

- [ ] Orden de locks consistente (no deadlock)
- [ ] RwLock: ¿read-heavy justifica RwLock sobre Mutex?
- [ ] Arc: ¿ciclos de referencia?
- [ ] Atomic: ¿ordenamiento correcto? (Acquire/Release/SeqCst)
- [ ] async: ¿Send + Sync implementados correctamente?

### 3e. Security Review

- [ ] Input validation en API pública (tipos Vanta*, connect())
- [ ] Path traversal en file operations (storage, backup)
- [ ] Secrets: ¿env vars o hardcode?
- [ ] Crypto: ¿algoritmos actualizados? (AES-256-GCM ok?)
- [ ] unsafe: ¿invariants documentados?

### 3f. Architecture Review

- [ ] ¿Single responsibility? (módulo hace una cosa)
- [ ] ¿Dependencias circulares? (codegraph para verificar)
- [ ] ¿Feature gating correcto? (no leakage entre features)
- [ ] ¿API pública semver-aware?
- [ ] ¿Archivos >500L candidatos a split?

### 3g. Testing Review

- [ ] ¿Tests unitarios para cada función pública?
- [ ] ¿Edge cases cubiertos? (vacíos, nulos, límites)
- [ ] ¿Tests de integración para flujos completos?
- [ ] ¿Property-based testing donde aplica?
- [ ] ¿Benchmarks para hot paths?

---

## FASE 4: Web Research

Para CADA hallazgo que no sea trivial, investigar:

```
MetaSearchMCP.search_web("<issue específico>")
→ Por ejemplo: "rust RwLock poisoned recover best practice 2026"
→ Por ejemplo: "HNSW ef_search auto-tuning PID loop"
→ Por ejemplo: "fjall column family compression performance"

Argus.extract_content("<url resultado>")
→ Documentación oficial
→ Stack Overflow / Discourse
→ Blog posts / RFCs
→ Crate docs / GitHub issues
```

Registrar:
- **Source URL** (documentación oficial o referencia)
- **Solución recomendada** (cita textual del source)
- **Alternativas consideradas** (si las hay)
- **Decisión final** (qué se va a hacer y por qué)

No investigar hallazgos triviales (typos, formato). Solo los que requieren
decisión técnica.

---

## FASE 5: Competitor Comparison

Por cada módulo, comparar con competidores relevantes:

| Competidor | Enfoque | Lo que tienen que nos falta | Lo que tenemos que les falta |
|-----------|---------|---------------------------|---------------------------|
| **Chroma** | Python-first, embedding store | HTTP API, rich client SDKs | WASM, Rust-native, MCP |
| **Pinecone** | Managed, SaaS | Auto-scaling, serverless | Open source, on-prem, WASM |
| **Qdrant** | Rust, gRPC, filtering | Geo-indexing, grouping | Multi-backend, WASM, MCP |
| **Milvus** | Distributed, GPU | Distributed, GPU acceleration | Single-node perf, WASM |
| **LanceDB** | Embedded, columnar | Columnar storage, multi-modal | Multi-backend, HNSW tuning |

Para cada hallazgo, preguntar:
- ¿La competencia tiene esto? → Feature gap
- ¿La competencia lo hace mejor? → Improvement opportunity
- ¿La competencia no lo tiene? → Competitive advantage, mantener
- ¿Hay benchmark públicos? → How do we compare?

---

## FASE 6: Triage → Backlog.md

Cada hallazgo pasa por este gate antes de agregarse al backlog:

```
Hallazgo: [issue concreto]
Archivo: path/to/file.rs:42
Tipo: (LOGIC/PATTERN/ARCH/CODE/ERROR/MISSING/FEATURE/ALGO/SECURITY/PERF)

Gate:
[ ] 1. ¿Es un issue real? (reproducible, no falso positivo)
[ ] 2. ¿Severidad real? (no inflada)
      🔴 Data loss, security, build broken → CRITICAL
      🟡 Feature broken, major perf regression → HIGH
      🔵 UX roto, CI warning, sin docs → MEDIUM
      ⚪ Code smell, mejora menor → LOW
      ℹ️ Observación, sugerencia → INFO
[ ] 3. ¿Impacto real en usuarios? (no teórico)
[ ] 4. ¿Esfuerzo estimado? (XS/S/M/L/XL)
[ ] 5. ¿Hay workaround conocido?
[ ] 6. ¿Relación con otros hallazgos? (duplicado, causa raíz)
[ ] 7. ¿Requiere feature flag o migración?

Output: ✅ BACKLOG | 🟢 FIX AHORA | ❌ DESCARTAR
```

Si el gate dice:
- **🟢 FIX AHORA** (esfuerzo XS-S, <30min, mismo módulo) → arreglar inmediatamente
- **✅ BACKLOG** → agregar a Backlog.md con ID único
- **❌ DESCARTAR** → registrar razón y no agregar

Formato para Backlog.md:

```markdown
| `DRV-XXX` | **Título** — Descripción breve | archivo:línea | 🟢 XS | 🟡 | ❌ |
```

ID: `DRV-<NNN>` (Deep ReView).

---

## FASE 7: Reporte del Módulo

Al completar cada módulo:

```markdown
## Módulo: ${MODULE} — Reporte

**Análisis:**
- Archivos analizados: N
- Líneas totales: N
- API pública: N funciones/structs
- Tests existentes: N

**Hallazgos:**
- Críticos (🔴): N
- Altos (🟡): N
- Medios (🔵): N
- Bajos (⚪): N
- Info (ℹ️): N

**Fixeados inmediatamente (🟢):** N
**Agregados a Backlog.md:** IDs: DRV-NNN, DRV-NNN...
**Descartados:** N

**Competitor gaps encontrados:** N
**Web research URLs:** N

**Próximo módulo:** ${NEXT_MODULE}
```

---

## Resumen Visual del Loop

```
/loop-goal --prompt-file .opencode/skills/review-deep/loop-prompt.md "MODULE=..." "DEPTH=full"
  │
  ├─ FASE 0: Skills (según tipo)
  ├─ FASE 1: codegraph → structural map
  ├─ FASE 2: Static analysis (tools)
  ├─ FASE 3: Deep review
  │   ├── 3a: Pattern scanning (expect, unsafe, etc.)
  │   ├── 3b: Error handling
  │   ├── 3c: Performance
  │   ├── 3d: Concurrency
  │   ├── 3e: Security
  │   ├── 3f: Architecture
  │   └── 3g: Testing
  ├─ FASE 4: Web research (por hallazgo no trivial)
  ├─ FASE 5: Competitor comparison
  ├─ FASE 6: Triage → fix ahora / backlog / descartar
  └─ FASE 7: Reporte + yield (siguiente módulo)
```
