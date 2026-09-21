# WSM-10 — Semántica score/distance consistente (3 transports)

## Metadata
- **Plan file:** docs/plans/2026-08-29-full-backlog-parallel.md (W12-1)
- **Creado:** 2026-08-30T18:45
- **last-synced:** 2026-08-30T18:45
- **Estado:** ⬜ PENDING

## Contexto

Investigación `research-vantadb-wasm-20260825` (H-15) y `research-vantadb-ts-20260825` (H-03)
detectaron divergencia en la semántica `score`/`distance` entre los 3 transports:

| Transport | API | Campo | Convención | Estado actual |
|---|---|---|---|---|
| Core Rust | `VantaMemorySearchHit.score` | `score` | **higher-is-better** (BM25/cosine similarity/RRF) | ✅ higher-is-better |
| Core Rust | `VantaSearchHit` (raw `search_vector`) | `distance` | **lower-is-better** (L2 / cosine distance) | ✅ lower-is-better |
| WASM binding (`search`) | `SearchHit.score` | `score` | higher-is-better (relevance) | ✅ correcto (lib.rs:1178) |
| WASM binding (`search_vector`) | `{node_id, score}` | **`score`** (label) | **lower-is-better** (valor = `distance`) | ⚠️ **mislabeled** (lib.rs:1250) |
| WASM binding (`similar_to_key`) | `SearchHit[]` | `score` | higher-is-better | ✅ (delega a `search`) |
| TS wrapper (`search`) | `SearchHit.distance` | `distance` | **lower-is-better** (wrapper invierte) | ✅ pinned CODE-091 |
| TS wrapper (`searchVector`) | `{node_id, distance}` | `distance` | lower-is-better | ✅ |
| Node (`search`) | `{record, score}` | `score` | higher-is-better (relevance) | ✅ correcto |
| Node (`search` docstring) | — | "by vector similarity" | — | ⚠️ dice "similarity" pero math es `1 - distance` |

**Bug real (WASM `search_vector`):** `lib.rs:1250` emite `hit.distance` (lower-is-better, distancia cruda)
bajo la key JS `"score"` — el .d.ts línea 579 dice `score` y la doc sería falsa. Tipo: mislabel semántico,
no afecta el orden de resultados (ya están ordenados por distancia ascendente en el core).

## Pre-mortem (del plan)

- **Fallo 1:** API TS ya fijada (CODE-091 preserve distance field) — no breaking → confirmado: docs-only
  + corrección no-breaking en WASM binding (renombrar `score` → `distance` en `search_vector`).
- **Fallo 2:** node usa "similarity" pero math es cosine (distance) — semántica → fix docstring node + doc.
- **Fallo 3:** docs pueden divergir entre 3 transports → mitigación: link cruzado en las 3 docs a la
  tabla canónica en TS_SDK.md.

## Stop conditions

>1d → docs-only por transport, core unificado en follow-up.

## Blast Radius

- `vantadb-wasm/src/lib.rs:1250` — línea que emite `score` cuando emite distancia; cambio no-breaking
  si el wrapper TS lee el field con el nombre nuevo. PERO: el TS wrapper `native.ts:365` lee `h.score`.
  Si renombramos el field WASM, tenemos que actualizar el wrapper TS también.
  **Decisión:** mejor renombrar a `distance` en WASM y ajustar TS wrapper (`native.ts`) — es la
  coherencia con `searchVector` que ya devuelve `{node_id, distance}`.
- `vantadb-wasm/src/vantadb_wasm.d.ts:579` — type signature.
- `vantadb-ts/src/native.ts:365` — wrapper adapter.
- `vantadb-ts/src/vantadb.ts:626,666,745,779` — consumers of `h.score`.
- `vantadb-node/src/lib.rs:179-181` — docstring "vector similarity" → matizar.
- `docs/api/WASM_STANDALONE.md`, `WASM_PERSISTENCE.md` (no tocar — no son API reference).
- `docs/api/TS_SDK.md` — sección "Distance vs Score (CODE-091)" ya existe; ampliar.
- `docs/api/NODE_SDK.md` — agregar nota simétrica.
- `docs/api/BINDINGS_NAMESPACES.md` — agregar nota de paridad score/distance (link cruzado).
- `vantadb-ts/src/__tests__/*.test.ts` — `dx04.test.ts:260` usa `expect(... .distance).toBeGreaterThanOrEqual(...)` — chequea `distance` ya (no toca).

## Contrato verificable

```
Select-String -Path "docs/api/WASM_API.md" -Pattern "score.*distance|distance.*score" | Measure-Object | Select-Object Count
```

> El path `docs/api/WASM_API.md` no existe en el repo. La doc canónica WASM
> vive en `vantadb-wasm/src/vantadb_wasm.d.ts` (binding types) + `docs/api/WASM_STANDALONE.md`
> (console runtime) + `docs/api/TS_SDK.md` (wrapper API). **Decisión:** creamos
> `docs/api/WASM_API.md` como índice canónico que re-exporta las tres piezas
> con la sección score/distance explícita. El contrato `>=1` se cumple porque
> la sección nueva contiene ambos términos.

### Contrato verificable (versión canónica, no rompedora)

```powershell
Select-String -Path "docs/api/WASM_API.md" -Pattern "score.*distance|distance.*score" | Measure-Object | Select-Object -ExpandProperty Count
```

Resultado esperado: `>=1` (la nueva sección "Score vs distance semantics" contiene ambas palabras).

## Spec

**Cambio 1 — Renombrar WASM `search_vector` field `score` → `distance` (no-breaking para TypeScript
porque el wrapper `native.ts` está dentro de nuestro control).**

Archivos:
- `vantadb-wasm/src/lib.rs:1250` — cambiar `"score"` por `"distance"`.
- `vantadb-wasm/src/vantadb_wasm.d.ts:579` — cambiar `score: number` por `distance: number`.
- `vantadb-ts/src/native.ts:365` — leer `h.distance` en lugar de `h.score` (en el path `search_vector`).
- `vantadb-ts/src/vantadb.ts:771-779` — ya devuelve `{node_id, distance}` (no toca).
- Tests E2E WASM: verificar que `search_vector` returns `{node_id, distance}[]`.

**Riesgo:** WASM `search` emite `score` (relevance, higher-is-better) y `search_vector` emite `distance`
(lower-is-better, raw). Diferentes APIs, diferentes campos. Coherente con la asimetría documentada en
TS_SDK.md (CODE-091). Coherente con `searchVector` en TS wrapper que ya devuelve `{node_id, distance}`.

**Cambio 2 — Doc-only fixes (sin tocar lógica):**

- `docs/api/WASM_API.md` (nuevo): índice WASM API con sección explícita score vs distance.
- `docs/api/NODE_SDK.md`: agregar nota simétrica a la tabla CODE-091 de TS_SDK.
- `docs/api/BINDINGS_NAMESPACES.md`: agregar nota de paridad con sección score/distance.
- `docs/api/TS_SDK.md`: ampliar sección existente (link cruzado a WASM/Node).
- `vantadb-node/src/lib.rs:179-181`: docstring "vector similarity" → "by relevance (BM25 / cosine
  similarity / RRF fused score; higher is better)" para alinear con semántica real.

## Steps

### Step 1: Renombrar WASM `search_vector` field (`score` → `distance`)
- **Archivos:**
  - `vantadb-wasm/src/lib.rs:1250`
  - `vantadb-wasm/src/vantadb_wasm.d.ts:579`
  - `vantadb-ts/src/native.ts:365`
  - `vantadb-ts/src/__tests__/subclients.test.ts` / `dx04.test.ts` (verificar)
- **Acción:**
  1. Cambiar `"score"` → `"distance"` en lib.rs:1250.
  2. Cambiar type `score: number` → `distance: number` en vantadb_wasm.d.ts:579.
  3. Actualizar JSDoc en línea 577.
  4. En native.ts línea ~365 (ruta `search_vector` wrapper), leer `h.distance` en lugar de `h.score`.
- **Verify:** `cargo check -p vantadb-wasm --target wasm32-unknown-unknown` exit 0 AND
  `cd vantadb-ts && npm run test 2>&1 | grep -i "searchVector"` sin regresiones.
- **Estado:** ⬜ PENDING

### Step 2: Fix node docstring semántica
- **Archivos:** `vantadb-node/src/lib.rs:179-181`
- **Acción:** cambiar docstring `/// Search memory records by vector similarity (with optional filters
  and text query). Returns hits ordered by relevance: each hit is
  \`{ record: MemoryRecord, score: number, explanation?: object }\`.`
  a:
  `/// Hybrid memory search. Returns hits ordered by **relevance** (higher score first): each hit is
  /// \`{ record: MemoryRecord, score: number, explanation?: object }\`.
  /// The \`score\` field is a relevance score — BM25 (text), cosine similarity, or RRF-fused
  /// (vector + text). It is **higher-is-better**, not a distance. Range and formula match
  /// the Rust core (\`src/sdk/serialization/vector_types.rs\`).`
- **Verify:** `cargo doc -p vantadb-node --no-deps` 0 warnings.
- **Estado:** ⬜ PENDING

### Step 3: Crear `docs/api/WASM_API.md` (índice canónico + sección score/distance)
- **Archivos:** `docs/api/WASM_API.md` (nuevo)
- **Acción:** crear el archivo con índice de los 3 doc layers (binding types / console runtime /
  wrapper) + sección "## Score vs distance semantics" que referencie CODE-091 en TS_SDK.md.
  El regex `score.*distance|distance.*score` debe aparecer en esta sección.
- **Verify:** el regex cuenta `>=1` líneas.
- **Estado:** ⬜ PENDING

### Step 4: Ampliar `TS_SDK.md` con link cruzado
- **Archivos:** `docs/api/TS_SDK.md:300-314`
- **Acción:** agregar nota que el wrapper TS mapea `h.score` (WASM search) → `hit.distance` (TS layer).
  Referenciar WASM_API.md y NODE_SDK.md.
- **Verify:** link no roto (Mermaid-style or plain text link).
- **Estado:** ⬜ PENDING

### Step 5: Agregar nota simétrica en `NODE_SDK.md`
- **Archivos:** `docs/api/NODE_SDK.md:170` (después del comentario `// hit: { record, score, ...}`)
- **Acción:** agregar nota que `score` es relevance (higher-is-better), diferencia con TS wrapper que
  usa `distance` (lower-is-better). Tabla CODE-091-style con las 3 filas (Node/TS/Python/Rust).
- **Verify:** markdown lints.
- **Estado:** ⬜ PENDING

### Step 6: Ampliar `BINDINGS_NAMESPACES.md` con nota de paridad score/distance
- **Archivos:** `docs/api/BINDINGS_NAMESPACES.md` (entre §"Domain Taxonomy" y §"SDK Surface
  Differences")
- **Acción:** agregar sección `## Score vs Distance Convention (CODE-091 / WSM-10)` con tabla compacta
  WASM/TS/Node/Python/HTTP.
- **Verify:** link cruzado a WASM_API.md funciona.
- **Estado:** ⬜ PENDING

### Step 7: Verify full + commit
- **Acción:**
  1. `cargo check -p vantadb-wasm --target wasm32-unknown-unknown` (validate renaming compiles)
  2. `cd vantadb-ts && npm run build 2>&1 | tail -20` (validar TS wrapper)
  3. PowerShell regex para contrato: `Select-String -Path "docs/api/WASM_API.md" -Pattern "score.*distance|distance.*score" | Measure-Object | Select-Object -ExpandProperty Count`
  4. `git add <solo los archivos tocados>` → commit `docs: WSM-10 — Score/distance semantics unificada (3 transports)`
- **Estado:** ⬜ PENDING

## Dependencias

- Ninguna externa.
- WSM-06 (completado, provee contexto de paridad WASM).

## Notas

- CODE-091 preserva el campo TS `distance` (no-breaking).
- Stop condition: si el renaming WASM rompe tests E2E → revertir Step 1, mantener solo docs-only.
- Pre-mortem Fallo 1 (TS pinned) — mitigado: cambio en WASM es upstream del wrapper TS, controlable.
- Pre-mortem Fallo 2 (node similarity vs cosine math) — mitigado: docstring explícita + sección docs.
- Pre-mortem Fallo 3 (docs divergentes) — mitigado: link cruzado WASM_API ↔ TS_SDK ↔ NODE_SDK ↔
  BINDINGS_NAMESPACES.

## SDP

SDP base: campaign-executor (auto via MCP), api-and-interface-design (cargada), source-driven-development
(falló — no se pudo cargar via tool skill por nombre incorrecto; equivalente: api-and-interface-design +
codebase-memory + lectura directa del código fuente), codebase-memory (disponible vía MCP).
Contrato keywords: ["score", "distance", "semantics", "WASM", "TS", "Node", "binding"].
Resultado SDP: 4 skills base relevantes (campaign-executor, api-and-interface-design, source-driven-dev,
codebase-memory). Limit maxSkills=8 respeta presupuesto.

## Context Save Point

(A llenar al cerrar step 7.)